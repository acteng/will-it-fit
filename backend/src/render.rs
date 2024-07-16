use anyhow::{bail, Result};
use geo::{BoundingRect, LineString};
use geojson::{Feature, GeoJson, Geometry};
use utils::{Mercator, OffsetCurve};

pub fn render_lanes(orig_wgs84: LineString, lanes: String) -> Result<String> {
    let mercator = Mercator::from(orig_wgs84.bounding_rect().unwrap()).unwrap();
    let orig = mercator.to_mercator(&orig_wgs84);

    let mut features = Vec::new();

    // TODO Make | be offset 0
    let mut width_sum = 0.0;
    for code in lanes.chars() {
        let (color, width) = match code {
            's' => ("grey", 2.0),
            'c' => ("green", 1.5),
            'b' => ("red", 3.25),
            'd' => ("black", 3.0),
            '|' => ("yellow", 0.5),
            _ => bail!("unknown lane code {code}"),
        };
        let Some(shifted) = orig.offset_curve(width_sum + width / 2.0) else {
            bail!("couldn't shift line");
        };
        // TODO buffer and make a polygon
        let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&shifted)));
        f.set_property("color", color);
        features.push(f);

        width_sum += width;
    }

    Ok(serde_json::to_string(&GeoJson::from(features))?)
}
