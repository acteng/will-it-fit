use anyhow::Result;
use geo::{BooleanOps, Polygon, Relate};

// TODO Would this work faster with planar coords?

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let mut polygons = read_polygons(&args[1])?;

    loop {
        println!("Have {} polygons", polygons.len());
        if let Some((idx1, idx2)) = find_first_touch(&polygons) {
            // Remove in the proper order
            let second = polygons.remove(idx2);
            let first = polygons.remove(idx1);
            polygons.extend(first.union(&second));
        } else {
            break;
        }

        if false && polygons.len() == 5600 {
            break;
        }
    }

    write_polygons("out.geojson", polygons)?;

    Ok(())
}

fn read_polygons(path: &str) -> Result<Vec<Polygon>> {
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

// Returns (idx1, idx2) such that idx1 < idx2
fn find_first_touch(polygons: &Vec<Polygon>) -> Option<(usize, usize)> {
    for idx1 in 0..polygons.len() {
        for idx2 in (idx1 + 1)..polygons.len() {
            let de9im = polygons[idx1].relate(&polygons[idx2]);
            if de9im.is_touches() {
                return Some((idx1, idx2));
            }
        }
    }
    None
}
