use std::collections::HashSet;

use anyhow::Result;
use geo::{BooleanOps, ChamberlainDuquetteArea, MultiPolygon, Polygon, Relate};
use indicatif::{ProgressBar, ProgressStyle};
use rstar::{primitives::GeomWithData, RTree, RTreeObject};
use union_find_rs::prelude::{DisjointSets, UnionFind};

// TODO Would this work faster with planar coords?

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let polygons = read_polygons(&args[1])?;
    let sets = find_all_adjacencies(&polygons);

    // TODO Take as param
    // TODO is the ChamberlainDuquetteArea approximation faster than the other trait? Are we
    // confident the polygons have the correct winding order?
    let max_unsigned_geodesic_area = 15_000.0;

    println!("Unioning {} sets", sets.len());
    let mut output = Vec::new();
    let progress = ProgressBar::new(sets.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());
    for set in sets {
        progress.inc(1);
        output.extend(union_all(&polygons, set, max_unsigned_geodesic_area));
    }
    progress.finish();
    println!("Result has {}", output.len());

    write_polygons("out.geojson", output)?;

    Ok(())
}

fn read_polygons(path: &str) -> Result<Vec<Polygon>> {
    println!("Reading {path}");
    let gj_string = std::fs::read_to_string(path)?;
    let gj: geojson::GeoJson = gj_string.parse()?;
    let mut polygons = Vec::new();
    for geom in geojson::quick_collection(&gj)? {
        if let geo::Geometry::Polygon(p) = geom {
            polygons.push(p);
        }
    }
    Ok(polygons)
}

fn write_polygons(path: &str, polygons: Vec<Polygon>) -> Result<()> {
    let gc = geo::GeometryCollection::from(polygons);
    let fc = geojson::FeatureCollection::from(&gc);
    std::fs::write(path, serde_json::to_string(&fc)?)?;
    Ok(())
}

// Returns disjoint sets of indices into polygons, where each set has polygons touching each other
fn find_all_adjacencies(polygons: &Vec<Polygon>) -> Vec<HashSet<usize>> {
    println!("Making rtree");
    // TODO Is the clone avoidable?
    let rtree = RTree::bulk_load(
        polygons
            .iter()
            .enumerate()
            .map(|(idx, p)| GeomWithData::new(p.clone(), idx))
            .collect(),
    );

    let progress = ProgressBar::new(polygons.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());

    let mut sets = DisjointSets::new();
    for i in 0..polygons.len() {
        sets.make_set(i).unwrap();
    }

    println!("Finding all adjacencies for {} polygons", polygons.len());
    for idx1 in 0..polygons.len() {
        progress.inc(1);
        for obj in rtree.locate_in_envelope_intersecting(&polygons[idx1].envelope()) {
            // TODO Can we use the geometry directly from GeomWithData?
            let idx2 = obj.data;
            if idx1 >= idx2 {
                continue;
            }
            let de9im = polygons[idx1].relate(&polygons[idx2]);
            if de9im.is_touches() {
                sets.union(&idx1, &idx2).unwrap();
            }
        }
    }
    progress.finish();
    sets.into_iter().collect()
}

// If the area isn't constrained, this should usually return exactly one polygon
fn union_all(
    polygons: &Vec<Polygon>,
    indices: HashSet<usize>,
    max_unsigned_geodesic_area: f64,
) -> Vec<Polygon> {
    let mut out = Vec::new();

    let indices: Vec<usize> = indices.into_iter().collect();
    let mut current = MultiPolygon::new(vec![polygons[indices[0]].clone()]);
    for idx in indices.into_iter().skip(1) {
        if current.chamberlain_duquette_unsigned_area() > max_unsigned_geodesic_area {
            out.extend(current.0);
            current = MultiPolygon::new(vec![polygons[idx].clone()]);
        } else {
            current = current.union(&MultiPolygon::new(vec![polygons[idx].clone()]));
        }
    }
    out.extend(current.0);

    out
}
