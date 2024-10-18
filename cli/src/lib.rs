// Classifies Inspire polygons into "road space" and "non-road space" polygons.
// Where necessary it will subdivide polygons into smaller polygons.
// It will also attempt to classify the negative space between polygons.


use std::collections::HashSet;

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct SourceInspirePolygons {
    geometry: geojson::Geometry,
}

pub fn classify_polygons_by_id(
    _polygons: Vec<SourceInspirePolygons>
) -> (Option<HashSet<usize>>, Option<HashSet<usize>>) {
    // TODO
    unimplemented!();
}