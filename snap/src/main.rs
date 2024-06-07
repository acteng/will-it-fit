use std::fs::File;
use std::io::BufReader;

use anyhow::{bail, Result};
use flatgeobuf::{
    FallibleStreamingIterator, FeatureProperties, FgbFeature, FgbReader, GeozeroGeometry,
};
use geo::{BoundingRect, Geometry, LineString};
use geojson::{de::deserialize_geometry, ser::serialize_geometry};
use serde::{Deserialize, Serialize};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let input_route = read_gj_input(&args[1])?;
    let raw_roads = read_nearby_roads(&input_route, &args[2])?;
    debug_roads(&raw_roads)?;

    Ok(())
}

fn read_gj_input(path: &str) -> Result<LineString> {
    #[derive(Deserialize)]
    struct Input {
        #[serde(deserialize_with = "deserialize_geometry")]
        geometry: LineString,
    }

    let gj_input = std::fs::read_to_string(path)?;
    let mut input: Vec<Input> = geojson::de::deserialize_feature_collection_str_to_vec(&gj_input)?;
    if input.len() != 1 {
        bail!("Expecting exactly one LineString, found {}", input.len());
    }
    Ok(input.pop().unwrap().geometry)
}

#[derive(Serialize)]
struct Road {
    #[serde(serialize_with = "serialize_geometry")]
    geometry: LineString,
    min_width: f64,
    avg_width: f64,
}

fn read_nearby_roads(route: &LineString, fgb_path: &str) -> Result<Vec<Road>> {
    let bbox = route.bounding_rect().unwrap();
    let mut file = BufReader::new(File::open(fgb_path)?);
    let mut fgb = FgbReader::open(&mut file)?.select_bbox(
        bbox.min().x,
        bbox.min().y,
        bbox.max().x,
        bbox.max().y,
    )?;

    let mut roads = Vec::new();
    // TODO Is there some serde magic?
    while let Some(feature) = fgb.next()? {
        roads.push(Road {
            geometry: get_linestring(feature)?,
            min_width: feature.property::<f64>("minimum").unwrap(),
            avg_width: feature.property::<f64>("average").unwrap(),
        });
    }
    Ok(roads)
}

fn get_linestring(f: &FgbFeature) -> Result<LineString> {
    let mut p = geozero::geo_types::GeoWriter::new();
    f.process_geom(&mut p)?;
    match p.take_geometry().unwrap() {
        Geometry::LineString(ls) => Ok(ls),
        _ => bail!("Wrong type in fgb"),
    }
}

fn debug_roads(roads: &Vec<Road>) -> Result<()> {
    println!(
        "{}",
        geojson::ser::to_feature_collection_string(roads)?.to_string()
    );
    Ok(())
}
