use std::sync::Once;

use anyhow::{bail, Result};
use flatgeobuf::{FgbFeature, GeozeroGeometry, HttpFgbReader};
use geo::{Line, LineString, Polygon, Rect};
use geojson::{de::deserialize_geometry, Feature, GeoJson, Geometry};

use serde::Deserialize;
use wasm_bindgen::prelude::*;

use utils::Mercator;
use widths::Timer;

mod render;

static START: Once = Once::new();

struct Features {
    features: Vec<Feature>,
}

impl widths::Output for Features {
    fn nearby_polygon(&mut self, mercator: &Mercator, polygon: &Polygon) {
        self.features
            .push(Feature::from(Geometry::from(&mercator.to_wgs84(polygon))));
    }
    fn perp_line(&mut self, mercator: &Mercator, line: Line, width: f64) {
        let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&line)));
        f.set_property("width", width);
        self.features.push(f);
    }
}

/// Takes GeoJSON with one LineString, and returns a FeatureCollection of all negative space
/// polygons in the polygon.
#[wasm_bindgen(js_name = getNegativeSpace)]
pub async fn get_negative_space(
    input: String,
    progress_cb: Option<js_sys::Function>,
) -> Result<String, JsValue> {
    // Panics shouldn't happen, but if they do, console.log them.
    console_error_panic_hook::set_once();
    START.call_once(|| {
        console_log::init_with_level(log::Level::Info).unwrap();
    });

    let input: Vec<Input> =
        geojson::de::deserialize_feature_collection_str_to_vec(&input).map_err(err_to_js)?;
    let linestrings: Vec<LineString> = input.into_iter().map(|x| x.geometry).collect();
    let input_route = linestrings[0].clone();

    let mut timer = Timer::new("calculate negative space", progress_cb);
    let step_size_meters = 5.0;
    let project_away_meters = 50.0;

    let bbox = widths::bbox(&input_route, project_away_meters);
    timer.step("Downloading nearby polygons");
    let url = "http://localhost:5173/out.fgb";
    let polygons = read_nearby_polygons(bbox, url).await.map_err(err_to_js)?;

    let mut out = Features {
        features: Vec::new(),
    };
    widths::calculate(
        &input_route,
        polygons,
        timer,
        step_size_meters,
        project_away_meters,
        &mut out,
    );

    serde_json::to_string(&GeoJson::from(out.features)).map_err(err_to_js)
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

async fn read_nearby_polygons(bbox: Rect, url: &str) -> Result<Vec<Polygon>> {
    let mut fgb = HttpFgbReader::open(url)
        .await?
        .select_bbox(bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y)
        .await?;

    let mut polygons = Vec::new();
    while let Some(feature) = fgb.next().await? {
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
