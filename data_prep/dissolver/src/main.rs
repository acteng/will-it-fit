use std::collections::HashSet;

use anyhow::Result;
use geo::{BooleanOps, MultiPolygon, Polygon, Relate};
use indicatif::{ProgressBar, ProgressStyle};
use union_find_rs::prelude::{DisjointSets, UnionFind};

// TODO Would this work faster with planar coords?

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let polygons = read_polygons(&args[1])?;
    let sets = find_all_adjacencies(&polygons);

    println!("Unioning {} sets", sets.len());
    let mut output = Vec::new();
    let progress = ProgressBar::new(sets.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());
    for set in sets {
        progress.inc(1);
        output.extend(union_all(&polygons, set));
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
    println!("Finding all adjacencies for {} polygons", polygons.len());
    let progress = ProgressBar::new(polygons.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());

    let mut sets = DisjointSets::new();
    for i in 0..polygons.len() {
        sets.make_set(i).unwrap();
    }

    // TODO rstar could help
    for idx1 in 0..polygons.len() {
        progress.inc(1);
        for idx2 in (idx1 + 1)..polygons.len() {
            let de9im = polygons[idx1].relate(&polygons[idx2]);
            if de9im.is_touches() {
                sets.union(&idx1, &idx2).unwrap();
            }
        }
    }
    progress.finish();
    sets.into_iter().collect()
}

// This should usually return exactly one polygon
fn union_all(polygons: &Vec<Polygon>, indices: HashSet<usize>) -> Vec<Polygon> {
    //println!("group with {:?}", indices);
    let indices: Vec<usize> = indices.into_iter().collect();
    let mut result = MultiPolygon::new(vec![polygons[indices[0]].clone()]);
    for idx in indices.into_iter().skip(1) {
        result = result.union(&MultiPolygon::new(vec![polygons[idx].clone()]));
    }
    //println!("  yielded {}", result.0.len());
    result.0
}
