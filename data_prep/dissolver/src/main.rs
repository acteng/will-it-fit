use std::collections::HashSet;

use anyhow::Result;
use geo::{BooleanOps, ChamberlainDuquetteArea, MultiPolygon, Polygon, Relate};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use rstar::{primitives::GeomWithData, ParentNode, RTree, RTreeNode, RTreeObject};
use union_find_rs::prelude::{DisjointSets, UnionFind};

static PROGRESS_STYLE: &str =
    "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})";

// TODO Would this work faster with planar coords?

/// This takes a .geojson file with polygons as input, then dissolves/merges adjacent polygons,
/// limited to a max_unsigned_geodesic_area. The input and output are TODO CRS.
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let polygons = read_polygons(&args[1])?;

    if true {
        let output = cascading_union(polygons);
        return write_polygons("out.geojson", output);
    }

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
    let mut fc = geojson::FeatureCollection::from(&gc);
    if false {
        fc.foreign_members = Some(
            serde_json::json!({
                "crs": { "type": "name", "properties": { "name": "urn:ogc:def:crs:EPSG::27700" } }
            })
            .as_object()
            .unwrap()
            .clone(),
        );
    }
    std::fs::write(path, serde_json::to_string(&fc)?)?;
    Ok(())
}

// Returns disjoint sets of indices into polygons, where each set has polygons touching each other
// TODO Could parallelize this too, since DisjointSets could be combined
// TODO We could limit area at this stage too, if we could keep a sum per disjoint set as we go,
// and refuse to join.
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
    // TODO Temporarily, give up when the input is too big
    if indices.len() > 1000 {
        println!("Skipping huge set with {}", indices.len());
        return indices
            .into_iter()
            .map(|idx| polygons[idx].clone())
            .collect();
    }

    let mut out = Vec::new();

    let mut current = MultiPolygon::new(vec![polygons[indices.pop().unwrap()].clone()]);
    while !indices.is_empty() {
        // If the current polygon is too big, start a new one
        if current.chamberlain_duquette_unsigned_area() > max_unsigned_geodesic_area {
            out.extend(current.0);
            current = MultiPolygon::new(vec![polygons[indices.pop().unwrap()].clone()]);
            // TODO Could we reuse the rtree here to prune the search for the huge sets?
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

fn cascading_union(polygons: Vec<Polygon>) -> Vec<Polygon> {
    let len = polygons.len();
    println!("Making rtree for {len} polygons");
    let rtree = RTree::bulk_load(polygons);

    println!("Cascading union");
    let progress = ProgressBar::new(len as u64)
        .with_style(ProgressStyle::with_template(PROGRESS_STYLE).unwrap());

    // From https://gist.github.com/urschrei/cd80b4d2ec3c75f12fa541a5bdbf6489
    let init = || MultiPolygon::<f64>::new(vec![]);
    let fold = |accum: MultiPolygon<f64>, poly: &Polygon<f64>| -> MultiPolygon<f64> {
        progress.inc(1);
        // NB the argument to union here is wrong / costly, because it won't accept &Polygon
        // Perhaps our current union method (which accepts &Self) is too strict?
        accum.union(&MultiPolygon::new(vec![poly.clone()]))
    };
    let reduce = |accum1: MultiPolygon<f64>, accum2: MultiPolygon<f64>| -> MultiPolygon<f64> {
        accum1.union(&accum2)
    };

    let result = bottom_up_fold_reduce(&rtree, init, fold, reduce).0;
    progress.finish();
    result
}

// From https://gist.github.com/urschrei/cd80b4d2ec3c75f12fa541a5bdbf6489
// TODO Try the rayon one
fn bottom_up_fold_reduce<T, S, I, F, R>(
    tree: &RTree<T>,
    mut init: I,
    mut fold: F,
    mut reduce: R,
) -> S
where
    T: RTreeObject,
    I: FnMut() -> S,
    F: FnMut(S, &T) -> S,
    R: FnMut(S, S) -> S,
{
    fn inner<T, S, I, F, R>(parent: &ParentNode<T>, init: &mut I, fold: &mut F, reduce: &mut R) -> S
    where
        T: RTreeObject,
        I: FnMut() -> S,
        F: FnMut(S, &T) -> S,
        R: FnMut(S, S) -> S,
    {
        parent
            .children()
            .iter()
            .fold(init(), |accum, child| match child {
                RTreeNode::Leaf(value) => fold(accum, value),
                RTreeNode::Parent(parent) => {
                    let value = inner(&parent, init, fold, reduce);

                    reduce(accum, value)
                }
            })
    }

    inner(tree.root(), &mut init, &mut fold, &mut reduce)
}
