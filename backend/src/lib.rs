use std::sync::Once;

use geo::LineString;
use geojson::de::deserialize_geometry;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

static START: Once = Once::new();

#[wasm_bindgen]
pub struct Backend {}

#[wasm_bindgen]
impl Backend {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Backend {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

        Backend {}
    }

    /// Takes GeoJSON with LineStrings
    #[wasm_bindgen()]
    pub fn query(&self, input: String) -> Result<String, JsValue> {
        let input: Vec<Input> =
            geojson::de::deserialize_feature_collection_str_to_vec(&input).map_err(err_to_js)?;
        let linestrings: Vec<LineString> = input.into_iter().map(|x| x.geometry).collect();

        Ok(format!("got {} linestrings", linestrings.len()))
    }
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[derive(Deserialize)]
struct Input {
    #[serde(deserialize_with = "deserialize_geometry")]
    geometry: LineString,
}
