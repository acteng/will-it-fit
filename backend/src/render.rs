use anyhow::{bail, Result};
use geo::{BoundingRect, LineString};
use geojson::{Feature, FeatureCollection, Geometry};
use utils::Mercator;

pub fn render_lanes(orig_wgs84: LineString, lanes: String) -> Result<String> {
    let mercator = Mercator::from(orig_wgs84.bounding_rect().unwrap()).unwrap();
    let orig = mercator.to_mercator(&orig_wgs84);

    // Use cavalier_contours to offset the original route linestring to draw lanes. First just
    // offset it for each lane edge.
    // TODO Make | be offset 0

    let mut lane_edges = vec![orig.clone()];
    let mut width_sum = 0.0;
    for code in lanes.chars() {
        let (_, width) = lane_config(code)?;
        width_sum += width;
        let Some(shifted) = offset_linestring(&orig, width_sum) else {
            bail!("couldn't shift line");
        };
        lane_edges.push(shifted);
    }

    let mut features = Vec::new();
    // Pairs of these lane edges can make polygons. This is better than buffering a
    // linestring centered in each lane, because corners ("bulges"?) match up better.
    for (pair, code) in lane_edges.windows(2).zip(lanes.chars()) {
        let (color, _) = lane_config(code)?;
        // Glue both edges together to make a polygon
        let mut pts = pair[0].0.clone();
        pts.reverse();
        pts.extend(pair[1].0.clone());
        let polygon = Polygon::new(LineString(pts), Vec::new());
        let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&polygon)));
        f.set_property("color", color);
        features.push(f);
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

fn lane_config(code: char) -> Result<(&'static str, f64)> {
    Ok(match code {
        's' => ("grey", 2.0),
        'c' => ("green", 1.5),
        'b' => ("red", 3.25),
        'd' => ("black", 3.0),
        '|' => ("yellow", 0.5),
        _ => bail!("unknown lane code {code}"),
    })
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
