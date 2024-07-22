use anyhow::{bail, Result};
use flatgeobuf::{FgbFeature, GeozeroGeometry, HttpFgbReader};
use geo::{
    BoundingRect, Coord, Densify, EuclideanLength, HaversineDestination, Line, LineIntersection,
    LineString, Point, Polygon, Rect,
};
use geojson::{Feature, GeoJson, Geometry};
use log::info;
use rstar::{primitives::GeomWithData, RTree, RTreeObject};
use utils::Mercator;

pub async fn calculate(route_wgs84: &LineString, url: &str) -> Result<String> {
    // TODO Take some params?
    let step_size_meters = 5.0;
    let project_away_meters = 50.0;

    // Increase the bounding box around the route by the max amount that we'll look away.
    let mut bbox = route_wgs84.bounding_rect().unwrap();
    // TODO This works in the UK, but make sure this is correct everywhere
    bbox.set_min(Point::from(bbox.min()).haversine_destination(135.0, project_away_meters));
    bbox.set_max(Point::from(bbox.max()).haversine_destination(45.0, project_away_meters));

    let mercator = Mercator::from(bbox).unwrap();

    let mut features = Vec::new();
    info!("Downloading nearby polygons");
    let polygons = read_nearby_polygons(bbox, url, &mercator).await?;
    // Debug them as GJ
    for p in &polygons {
        features.push(Feature::from(Geometry::from(&mercator.to_wgs84(p))));
    }

    info!("Making rtree of {} polygons", polygons.len());
    let rtree = RTree::bulk_load(
        polygons
            .iter()
            .enumerate()
            .map(|(idx, p)| GeomWithData::new(p.clone(), idx))
            .collect(),
    );

    info!("Calculating perpendiculars");
    let mut num_perps = 0;
    let mut num_hit_checks = 0;
    for (pt, angle) in points_along_line(&mercator.to_mercator(route_wgs84), step_size_meters) {
        num_perps += 1;
        let mut test_lines = Vec::new();
        for angle_offset in [-90.0, 90.0] {
            let projected = project_away(pt, angle + angle_offset, project_away_meters);
            let full_line = Line::new(pt, projected);

            test_lines.extend(shortest_line_hitting_polygon(
                full_line,
                &polygons,
                &rtree,
                &mut num_hit_checks,
            ));
        }
        // If either of the test lines doesn't hit anything within project_away_meters, then
        // something's probably wrong -- skip it as output
        if test_lines.len() != 2 {
            continue;
        }
        let full_line = Line::new(test_lines[0].end, test_lines[1].end);
        let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&full_line)));
        f.set_property("width", full_line.euclidean_length());
        features.push(f);
    }
    info!(
        "Tried {} perpendiculars, with a total of {} line hit checks",
        num_perps, num_hit_checks
    );

    Ok(serde_json::to_string(&GeoJson::from(features))?)
}

async fn read_nearby_polygons(bbox: Rect, url: &str, mercator: &Mercator) -> Result<Vec<Polygon>> {
    let mut fgb = HttpFgbReader::open(url)
        .await?
        .select_bbox(bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y)
        .await?;

    let mut polygons = Vec::new();
    while let Some(feature) = fgb.next().await? {
        polygons.push(mercator.to_mercator(&get_polygon(feature)?));
    }
    Ok(polygons)
}

// TODO multipolygons?
fn get_polygon(f: &FgbFeature) -> Result<Polygon> {
    let mut p = geozero::geo_types::GeoWriter::new();
    f.process_geom(&mut p)?;
    match p.take_geometry().unwrap() {
        geo::Geometry::Polygon(p) => Ok(p),
        _ => bail!("Wrong type in fgb"),
    }
}

// Every step_size along a LineString, returns the point and angle
fn points_along_line(linestring: &LineString, step_size_meters: f64) -> Vec<(Coord, f64)> {
    let mut result = Vec::new();
    // Using lines instead of coords so we can get the angle -- but is this hard to reason about?
    for line in linestring.densify(step_size_meters).lines() {
        // TODO For the last line, use the last point too
        let pt = line.start;
        let angle = line_angle_degrees(line);
        result.push((pt, angle));
    }
    result
}

fn line_angle_degrees(line: Line) -> f64 {
    line.dy().atan2(line.dx()).to_degrees()
}

fn project_away(pt: Coord, angle_degrees: f64, distance: f64) -> Coord {
    let (sin, cos) = angle_degrees.to_radians().sin_cos();
    Coord {
        x: pt.x + distance * cos,
        y: pt.y + distance * sin,
    }
}

// Assuming line.start is outside all of the polygons, looks for all possible intersections between
// the line and a polygon, and trims the line back to the edge of the nearest polygon
fn shortest_line_hitting_polygon(
    line: Line,
    polygons: &Vec<Polygon>,
    rtree: &RTree<GeomWithData<Polygon, usize>>,
    num_hit_checks: &mut usize,
) -> Option<Line> {
    let mut shortest: Option<(Line, f64)> = None;
    for obj in rtree.locate_in_envelope_intersecting(&line.envelope()) {
        // Ignore polygon holes
        for polygon_line in polygons[obj.data].exterior().lines() {
            *num_hit_checks += 1;
            if let Some(LineIntersection::SinglePoint { intersection, .. }) =
                geo::algorithm::line_intersection::line_intersection(line, polygon_line)
            {
                let candidate = Line::new(line.start, intersection);
                let candidate_length = candidate.euclidean_length();
                if shortest
                    .as_ref()
                    .map(|(_, len)| candidate_length < *len)
                    .unwrap_or(true)
                {
                    shortest = Some((candidate, candidate_length));
                }
            }
        }
    }
    shortest.map(|pair| pair.0)
}
