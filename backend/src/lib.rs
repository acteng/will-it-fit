use std::sync::Once;

use anyhow::{bail, Result};
use flatgeobuf::{FeatureProperties, FgbFeature, GeozeroGeometry, HttpFgbReader};
use geo::{BoundingRect, Geometry, HaversineLength, LineString};
use geojson::de::deserialize_geometry;

use serde::Deserialize;
use wasm_bindgen::prelude::*;

use crate::network::{Network, Road};

mod network;
mod render;

static START: Once = Once::new();

/// Takes GeoJSON with one LineString, snaps to the network, and returns a FeatureCollection
/// with width breakdowns
#[wasm_bindgen(js_name = snapRoads)]
pub async fn snap_roads(input: String) -> Result<String, JsValue> {
    // Panics shouldn't happen, but if they do, console.log them.
    console_error_panic_hook::set_once();
    START.call_once(|| {
        console_log::init_with_level(log::Level::Info).unwrap();
    });

    let input: Vec<Input> =
        geojson::de::deserialize_feature_collection_str_to_vec(&input).map_err(err_to_js)?;
    let linestrings: Vec<LineString> = input.into_iter().map(|x| x.geometry).collect();
    let input_route = linestrings[0].clone();

    let network = read_nearby_roads(&input_route, "http://localhost:5173/road_widths.fgb")
        .await
        .map_err(err_to_js)?;
    network.snap_route(input_route).map_err(err_to_js)
}

/// Takes GeoJSON with one LineString, and returns a FeatureCollection of all roads in the
/// network in the bounding box. Use to debug why a route isn't found.
#[wasm_bindgen(js_name = debugRoads)]
pub async fn debug_roads(input: String) -> Result<String, JsValue> {
    // Panics shouldn't happen, but if they do, console.log them.
    console_error_panic_hook::set_once();
    START.call_once(|| {
        console_log::init_with_level(log::Level::Info).unwrap();
    });

    let input: Vec<Input> =
        geojson::de::deserialize_feature_collection_str_to_vec(&input).map_err(err_to_js)?;
    let linestrings: Vec<LineString> = input.into_iter().map(|x| x.geometry).collect();
    let input_route = linestrings[0].clone();

    let network = read_nearby_roads(&input_route, "http://localhost:5173/road_widths.fgb")
        .await
        .map_err(err_to_js)?;
    network.debug_roads().map_err(err_to_js)
}

/// Takes GeoJSON with one LineString and a string representing lane config, and returns a FeatureCollection
/// with polygons per lane
#[wasm_bindgen(js_name = renderLanes)]
pub fn render_lanes(input_gj: String, lanes: String) -> Result<String, JsValue> {
    // Panics shouldn't happen, but if they do, console.log them.
    console_error_panic_hook::set_once();
    START.call_once(|| {
        console_log::init_with_level(log::Level::Info).unwrap();
    });

    // TODO take just one feature
    let input: Vec<Input> =
        geojson::de::deserialize_feature_collection_str_to_vec(&input_gj).map_err(err_to_js)?;
    let linestrings: Vec<LineString> = input.into_iter().map(|x| x.geometry).collect();
    let input_route = linestrings[0].clone();

    render::render_lanes(input_route, lanes).map_err(err_to_js)
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[derive(Deserialize)]
struct Input {
    #[serde(deserialize_with = "deserialize_geometry")]
    geometry: LineString,
}

async fn read_nearby_roads(route: &LineString, url: &str) -> Result<Network> {
    let bbox = route.bounding_rect().unwrap();
    let mut fgb = HttpFgbReader::open(url)
        .await?
        .select_bbox(bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y)
        .await?;

    let mut network = Network::new();
    // TODO Is there some serde magic?
    while let Some(feature) = fgb.next().await? {
        let geometry = get_linestring(feature)?;
        let length = geometry.haversine_length();
        network.add_road(Road {
            geometry,
            min_width: feature.property::<f64>("minimum").unwrap(),
            avg_width: feature.property::<f64>("average").unwrap(),
            length,
        });
    }
    Ok(network)
}

fn get_linestring(f: &FgbFeature) -> Result<LineString> {
    let mut p = geozero::geo_types::GeoWriter::new();
    f.process_geom(&mut p)?;
    match p.take_geometry().unwrap() {
        Geometry::LineString(ls) => Ok(ls),
        _ => bail!("Wrong type in fgb"),
    }
}
