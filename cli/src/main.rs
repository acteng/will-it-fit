use std::fs::File;
use std::io::BufReader;

use anyhow::{bail, Result};
use flatgeobuf::{FallibleStreamingIterator, FgbFeature, FgbReader, GeozeroGeometry};
use geo::{Line, Polygon, Rect};
use geojson::{Feature, GeoJson, Geometry};
use utils::{Mercator, Tags};

use widths::Timer;

fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Call with a .osm.pbf or .osm.xml input");
        std::process::exit(1);
    }

    let graph = utils::osm2graph::Graph::new(
        &std::fs::read(&args[1])?,
        keep_edge,
        &mut utils::osm2graph::NullReader,
    )?;

    let mut out = Features {
        features: Vec::new(),
        widths: Vec::new(),
    };

    for (idx, edge) in graph.edges.iter().enumerate() {
        println!("Working on edge {}/{}", idx, graph.edges.len());

        let edge_wgs84 = graph.mercator.to_wgs84(&edge.linestring);
        let mut timer = Timer::new("calculate negative space", None);
        let step_size_meters = 5.0;
        let project_away_meters = 50.0;

        let bbox = widths::bbox(&edge_wgs84, project_away_meters);
        timer.step("Downloading nearby polygons");
        let polygons = read_nearby_polygons(bbox, "../web/public/out.fgb")?;

        widths::calculate(
            &edge_wgs84,
            polygons,
            timer,
            step_size_meters,
            project_away_meters,
            &mut out,
        );

        let mut sum = 0.0;
        let mut min = f64::MAX;
        let n = out.widths.len();
        for width in out.widths.drain(..) {
            sum += width;
            min = min.min(width);
        }
        let mut f = Feature::from(Geometry::from(&edge_wgs84));
        f.set_property("min_width", min);
        f.set_property("avg_width", sum / (n as f64));
        out.features.push(f);
    }

    Ok(std::fs::write(
        "out.geojson",
        serde_json::to_string(&GeoJson::from(out.features))?,
    )?)
}

fn keep_edge(tags: &Tags) -> bool {
    if !tags.has("highway") || tags.is("highway", "proposed") || tags.is("area", "yes") {
        return false;
    }
    true
}

fn read_nearby_polygons(bbox: Rect, path: &str) -> Result<Vec<Polygon>> {
    // TODO Open once?
    let mut fgb = FgbReader::open(BufReader::new(File::open(path)?))?.select_bbox(
        bbox.min().x,
        bbox.min().y,
        bbox.max().x,
        bbox.max().y,
    )?;

    let mut polygons = Vec::new();
    while let Some(feature) = fgb.next()? {
        polygons.push(get_polygon(feature)?);
    }
    Ok(polygons)
}

fn get_polygon(f: &FgbFeature) -> Result<Polygon> {
    let mut p = geozero::geo_types::GeoWriter::new();
    f.process_geom(&mut p)?;
    match p.take_geometry().unwrap() {
        geo::Geometry::Polygon(p) => Ok(p),
        _ => bail!("Wrong type in fgb"),
    }
}

struct Features {
    features: Vec<Feature>,
    widths: Vec<f64>,
}

impl widths::Output for Features {
    fn nearby_polygon(&mut self, _: &Mercator, _: &Polygon) {}
    fn perp_line(&mut self, mercator: &Mercator, line: Line, width: f64) {
        let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&line)));
        f.set_property("width", width);
        self.features.push(f);

        self.widths.push(width);
    }
}
