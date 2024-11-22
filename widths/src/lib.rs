use geo::{
    BoundingRect, Coord, Densify, Destination, Euclidean, Haversine, Length, Line,
    LineIntersection, LineString, Point, Polygon, Rect,
};
use log::info;
use rstar::{primitives::GeomWithData, RTree, RTreeObject};
use utils::Mercator;

pub use crate::timer::Timer;

mod timer;

pub fn bbox(route_wgs84: &LineString, project_away_meters: f64) -> Rect {
    // Increase the bounding box around the route by the max amount that we'll look away.
    let bbox = route_wgs84.bounding_rect().unwrap();
    // TODO This works in the UK, but make sure this is correct everywhere
    let min = Haversine::destination(Point::from(bbox.min()), 135.0, project_away_meters).into();
    let max = Haversine::destination(Point::from(bbox.max()), 45.0, project_away_meters).into();

    // TODO Var names above aren't always true, so do this to be safe
    LineString::new(vec![min, max]).bounding_rect().unwrap()
}

pub trait Output {
    fn nearby_polygon(&mut self, mercator: &Mercator, polygon: &Polygon);
    fn perp_line(&mut self, mercator: &Mercator, line: Line, width: f64);
}

// TODO docs
// everything wgs84 as input
pub fn calculate<O: Output>(
    route_wgs84: &LineString,
    mut polygons: Vec<Polygon>,
    mut timer: Timer,
    step_size_meters: f64,
    project_away_meters: f64,
    output: &mut O,
) {
    let mercator = Mercator::from(bbox(route_wgs84, project_away_meters)).unwrap();

    for p in &mut polygons {
        mercator.to_mercator_in_place(p);
        output.nearby_polygon(&mercator, p);
    }

    timer.step(format!("Making rtree of {} polygons", polygons.len()));
    let rtree = RTree::bulk_load(
        polygons
            .iter()
            .enumerate()
            .map(|(idx, p)| GeomWithData::new(p.clone(), idx))
            .collect(),
    );

    let test_points = points_along_line(&mercator.to_mercator(route_wgs84), step_size_meters);
    let num_test_points = test_points.len();
    timer.push(format!(
        "Calculating perpendiculars at {num_test_points} points"
    ));
    let mut num_perps = 0;
    let mut num_hit_checks = 0;
    for (pt, angle) in test_points {
        num_perps += 1;
        // TODO Proper Timer API for this
        if num_perps % 200 == 0 {
            timer.step(format!(
                "{}% perpendiculars done",
                ((num_perps as f64 / num_test_points as f64) * 100.0).round()
            ));
        }

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
        output.perp_line(&mercator, full_line, full_line.length::<Euclidean>());
    }
    timer.pop();
    info!(
        "Tried {} perpendiculars, with a total of {} line hit checks",
        num_perps, num_hit_checks
    );
    timer.done();
}

// Every step_size along a LineString, returns the point and angle
fn points_along_line(linestring: &LineString, step_size_meters: f64) -> Vec<(Coord, f64)> {
    let mut result = Vec::new();
    // Using lines instead of coords so we can get the angle -- but is this hard to reason about?
    for line in linestring.densify::<Euclidean>(step_size_meters).lines() {
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
                let candidate_length = candidate.length::<Euclidean>();
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
