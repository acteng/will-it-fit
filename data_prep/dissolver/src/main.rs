use std::collections::HashSet;

use anyhow::Result;
use geo::{BooleanOps, ChamberlainDuquetteArea, MultiPolygon, Polygon, Relate};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use rstar::{primitives::GeomWithData, RTree, RTreeObject};
use union_find_rs::prelude::{DisjointSets, UnionFind};

static PROGRESS_STYLE: &str =
    "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})";

// TODO Would this work faster with planar coords?

/// This takes a .geojson file with polygons as input, then dissolves/merges adjacent polygons,
/// limited to a max_unsigned_geodesic_area. The input and output are WGS84.
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let polygons = read_polygons(&args[1])?;
    let sets = find_all_adjacencies(&polygons);

    // TODO Take as param
    // TODO is the ChamberlainDuquetteArea approximation faster than the other trait? Are we
    // confident the polygons have the correct winding order?
    let max_unsigned_geodesic_area = 15_000.0;

    println!("Unioning {} sets", sets.len());
    let output: Vec<Polygon> = sets
        .into_par_iter()
        .progress_with_style(ProgressStyle::with_template(PROGRESS_STYLE).unwrap())
        .flat_map(|set| {
            union_all(
                &polygons,
                set.into_iter().collect(),
                max_unsigned_geodesic_area,
            )
        })
        .collect();

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
// TODO Could parallelize this too, since DisjointSets could be combined
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

    let mut sets = DisjointSets::new();
    for i in 0..polygons.len() {
        sets.make_set(i).unwrap();
    }

    println!("Finding all adjacencies for {} polygons", polygons.len());
    let progress = ProgressBar::new(polygons.len() as u64)
        .with_style(ProgressStyle::with_template(PROGRESS_STYLE).unwrap());
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
    mut indices: Vec<usize>,
    max_unsigned_geodesic_area: f64,
) -> Vec<Polygon> {
    let mut out = Vec::new();

    let mut current = MultiPolygon::new(vec![polygons[indices.pop().unwrap()].clone()]);
    while !indices.is_empty() {
        // If the current polygon is too big, start a new one
        if current.chamberlain_duquette_unsigned_area() > max_unsigned_geodesic_area {
            out.extend(current.0);
            current = MultiPolygon::new(vec![polygons[indices.pop().unwrap()].clone()]);
        } else if let Some(idx_of_idx) = indices
            .iter()
            .position(|idx| current.relate(&polygons[*idx]).is_touches())
        {
            // Union the current polygon with something else
            // (Maybe a Vec of indices is a weird data structure)
            let idx = indices.remove(idx_of_idx);
            current = current.union(&MultiPolygon::new(vec![polygons[idx].clone()]));
        } else {
            // The current polygon doesn't touch anything, so start a new group
            out.extend(current.0);
            current = MultiPolygon::new(vec![polygons[indices.pop().unwrap()].clone()]);
        }
    }
    out.extend(current.0);

    out
}
