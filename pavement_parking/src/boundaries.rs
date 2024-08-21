use std::collections::HashMap;
use std::io::BufWriter;

use anyhow::Result;
use fs_err::File;
use geo::{Contains, LineString, MultiPolygon, Polygon};
use geojson::{Feature, FeatureCollection, FeatureWriter, Value};
use rstar::{primitives::GeomWithData, RTree, RTreeObject};

use crate::Rating;

/// Aggregate counts per LAD and CA boundaries
pub struct Boundaries {
    rtree: RTree<GeomWithData<Polygon, String>>,
    // Per boundary name, the count for [red, amber, green]
    counts: HashMap<String, [usize; 3]>,
}

impl Boundaries {
    pub fn load(path: &str) -> Result<Self> {
        println!("Reading boundaries from {path}");
        let gj: FeatureCollection = fs_err::read_to_string(path)?.parse()?;
        let mut boundaries = Vec::new();
        let mut counts = HashMap::new();
        for f in gj.features {
            let name = f.property("name").unwrap().as_str().unwrap().to_string();
            let mp: MultiPolygon =
                if matches!(f.geometry.as_ref().unwrap().value, Value::Polygon(_)) {
                    MultiPolygon(vec![f.try_into()?])
                } else {
                    f.try_into()?
                };
            // MultiPolygon isn't supported, so just insert multiple names
            for polygon in mp {
                boundaries.push(GeomWithData::new(polygon, name.clone()));
            }
            counts.insert(name, [0, 0, 0]);
        }
        println!("Building RTree of {} boundaries", boundaries.len());
        let rtree = RTree::bulk_load(boundaries);

        println!("");
        Ok(Self { rtree, counts })
    }

    pub fn handle_road(&mut self, geom: &LineString, rating: Rating) {
        // Find all matching boundaries
        for obj in self.rtree.locate_in_envelope_intersecting(&geom.envelope()) {
            // TODO Or even just intersects, to handle boundaries?
            if obj.geom().contains(geom) {
                let idx = match rating {
                    Rating::Red => 0,
                    Rating::Amber => 1,
                    Rating::Green => 2,
                };
                self.counts.get_mut(&obj.data).unwrap()[idx] += 1;
            }
        }
    }

    /// Write all boundaries with non-zero counts
    pub fn write_output(mut self, path: &str) -> Result<()> {
        let mut summaries_writer = FeatureWriter::from_writer(BufWriter::new(File::create(path)?));
        for obj in self.rtree.drain() {
            let mut f = Feature::from(Value::from(obj.geom()));
            let counts = self.counts[&obj.data];
            if counts[0] + counts[1] + counts[2] > 0 {
                f.set_property("name", obj.data);
                f.set_property("red", counts[0]);
                f.set_property("amber", counts[1]);
                f.set_property("green", counts[2]);
                summaries_writer.write_feature(&f)?;
            }
        }
        Ok(())
    }
}
