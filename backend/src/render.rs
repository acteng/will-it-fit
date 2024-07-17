use anyhow::{bail, Result};
use geo::{BoundingRect, LineString};
use geojson::{Feature, FeatureCollection, Geometry};
use utils::Mercator;

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
        let Some(shifted) = offset_linestring(&orig, width_sum + width / 2.0) else {
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

use cavalier_contours::polyline::{
    PlineCreation, PlineOffsetOptions, PlineSource, PlineVertex, Polyline,
};
use geo::{Coord, Polygon};

fn offset_linestring(linestring: &LineString, width: f64) -> Option<LineString> {
    let pl = linestring_to_pline(linestring);
    let opts = PlineOffsetOptions {
        handle_self_intersects: true,
        ..Default::default()
    };
    let result = pl.parallel_offset_opt(width, &opts);
    if result.len() != 1 {
        log::warn!("Unexpected number of results {}", result.len());
    }
    if result.is_empty() {
        return None;
    }
    Some(pline_to_linestring(&result[0]))
}

fn linestring_to_pline(linestring: &LineString) -> Polyline {
    let is_closed = false;
    Polyline::from_iter(
        linestring
            .0
            .iter()
            .map(|pt| PlineVertex::new(pt.x, pt.y, 0.0)),
        is_closed,
    )
}

fn pline_to_linestring(pline: &Polyline) -> LineString {
    LineString::new(
        pline
            .vertex_data
            .iter()
            .map(|v| Coord { x: v.x, y: v.y })
            .collect(),
    )
}

fn buffer_linestring(
    linestring: &LineString,
    left_meters: f64,
    right_meters: f64,
) -> Option<Polygon> {
    assert!(left_meters >= 0.0);
    assert!(right_meters >= 0.0);
    let left = offset_linestring(linestring, -left_meters)?;
    let right = offset_linestring(linestring, right_meters)?;
    // Make a polygon by gluing these points together
    let mut pts = left.0;
    pts.reverse();
    pts.extend(right.0);
    Some(Polygon::new(LineString(pts), Vec::new()))
}
