use anyhow::{bail, Result};
use geo::{BoundingRect, LineString};
use geojson::{Feature, FeatureCollection, Geometry};
use utils::{buffer_linestring, Mercator, OffsetCurve};

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
        let Some(thickened) = buffer_linestring(&shifted, width / 2.0, width / 2.0) else {
            bail!("couldn't thicken lane");
        };
        let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&thickened)));
        f.set_property("color", color);
        features.push(f);

        width_sum += width;
    }

    let fc = FeatureCollection {
        features,
        bbox: None,
        foreign_members: Some(
            serde_json::json!({
                "width": width_sum,
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };
    Ok(serde_json::to_string(&fc)?)
}
