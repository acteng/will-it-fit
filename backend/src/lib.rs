use std::sync::Once;

use anyhow::Result;
use geo::LineString;
use geojson::de::deserialize_geometry;

use serde::Deserialize;
use wasm_bindgen::prelude::*;

mod negative_space;
mod render;

static START: Once = Once::new();

/// Takes GeoJSON with one LineString, and returns a FeatureCollection of all negative space
/// polygons in the polygon.
#[wasm_bindgen(js_name = getNegativeSpace)]
pub async fn get_negative_space(input: String) -> Result<String, JsValue> {
    // Panics shouldn't happen, but if they do, console.log them.
    console_error_panic_hook::set_once();
    START.call_once(|| {
        console_log::init_with_level(log::Level::Info).unwrap();
    });

    let input: Vec<Input> =
        geojson::de::deserialize_feature_collection_str_to_vec(&input).map_err(err_to_js)?;
    let linestrings: Vec<LineString> = input.into_iter().map(|x| x.geometry).collect();
    let input_route = linestrings[0].clone();

    negative_space::calculate(&input_route, "http://localhost:5173/topo_areas.fgb")
        .await
        .map_err(err_to_js)
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

#[derive(Deserialize)]
struct Input {
    #[serde(deserialize_with = "deserialize_geometry")]
    geometry: LineString,
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}
