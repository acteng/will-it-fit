use std::fs::File;
use std::io::BufReader;

use actix_cors::Cors;
use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use anyhow::{bail, Result};
use flatgeobuf::{
    FallibleStreamingIterator, FeatureProperties, FgbFeature, FgbReader, GeozeroGeometry,
};
use geo::{BoundingRect, Geometry, HaversineLength, LineString};
use geojson::de::deserialize_geometry;
use serde::Deserialize;

use crate::network::{Network, Road};

mod network;

#[post("/")]
async fn snap(req_body: String) -> impl Responder {
    match (|| {
        let input_route = read_gj_input(req_body)?;
        let network = read_nearby_roads(&input_route, "/home/dabreegster/road_widths.fgb")?;
        network.snap_route(input_route)
    })() {
        Ok(resp) => HttpResponse::Ok().body(resp),
        // TODO Maybe an error code would be nicer
        Err(err) => {
            HttpResponse::Ok().body(serde_json::json!({"error": err.to_string()}).to_string())
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().wrap(Cors::permissive()).service(snap))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

fn read_gj_input(gj_input: String) -> Result<LineString> {
    #[derive(Deserialize)]
    struct Input {
        #[serde(deserialize_with = "deserialize_geometry")]
        geometry: LineString,
    }

    let mut input: Vec<Input> = geojson::de::deserialize_feature_collection_str_to_vec(&gj_input)?;
    if input.len() != 1 {
        bail!("Expecting exactly one LineString, found {}", input.len());
    }
    Ok(input.pop().unwrap().geometry)
}

fn read_nearby_roads(route: &LineString, fgb_path: &str) -> Result<Network> {
    let bbox = route.bounding_rect().unwrap();
    let mut file = BufReader::new(File::open(fgb_path)?);
    let mut fgb = FgbReader::open(&mut file)?.select_bbox(
        bbox.min().x,
        bbox.min().y,
        bbox.max().x,
        bbox.max().y,
    )?;

    let mut network = Network::new();
    // TODO Is there some serde magic?
    while let Some(feature) = fgb.next()? {
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
