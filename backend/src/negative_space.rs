use anyhow::{bail, Result};
use flatgeobuf::{FeatureProperties, FgbFeature, GeozeroGeometry, HttpFgbReader};
use geo::{BoundingRect, Polygon, LineString};
use geojson::{Feature, GeoJson, Geometry};

pub async fn calculate(route: &LineString, url: &str) -> Result<String> {
    let polygons = read_nearby_polygons(route, url).await?;

    // Debug them as GJ
    let mut features = Vec::new();
    for (p, style) in polygons {
        let mut f = Feature::from(Geometry::from(&p));
        f.set_property("style", style);
        features.push(f);
    }
    Ok(serde_json::to_string(&GeoJson::from(features))?)
}

async fn read_nearby_polygons(route: &LineString, url: &str) -> Result<Vec<(Polygon, String)>> {
    let bbox = route.bounding_rect().unwrap();
    let mut fgb = HttpFgbReader::open(url)
        .await?
        .select_bbox(bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y)
        .await?;

    let mut polygons = Vec::new();
    while let Some(feature) = fgb.next().await? {
        let style = feature.property::<String>("style_description").unwrap();
        if keep_polygon(&style) {
            let geometry = get_polygon(feature)?;
            polygons.push((geometry, style));
        }
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

// TODO Do this filtering to the input
fn keep_polygon(style: &str) -> bool {
    !matches!(
        style,
        "Road Or Track Fill" | "Roadside Manmade Fill" | "Path Fill" | "Traffic Calming Fill"
    )
}
