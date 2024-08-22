use std::collections::HashMap;
use std::io::BufWriter;

use anyhow::Result;
use enum_map::EnumMap;
use fs_err::File;
use geo::{Contains, MultiPolygon, Polygon};
use geojson::{Feature, FeatureCollection, FeatureWriter, Value};
use rstar::{primitives::GeomWithData, RTree, RTreeObject};

use crate::{Rating, Road};

/// Aggregate counts per LAD and CA boundaries
pub struct Boundaries {
    rtree: RTree<GeomWithData<Polygon, String>>,
    info: HashMap<String, Boundary>,
}

struct Boundary {
    counts: EnumMap<Rating, usize>,
    total_length: EnumMap<Rating, f64>,
}

impl Boundaries {
    pub fn load(path: &str) -> Result<Self> {
        println!("Reading boundaries from {path}");
        let gj: FeatureCollection = fs_err::read_to_string(path)?.parse()?;
        let mut boundaries = Vec::new();
        let mut info = HashMap::new();
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
            info.insert(
                name,
                Boundary {
                    counts: EnumMap::default(),
                    total_length: EnumMap::default(),
                },
            );
        }
        println!("Building RTree of {} boundaries", boundaries.len());
        let rtree = RTree::bulk_load(boundaries);

        println!("");
        Ok(Self { rtree, info })
    }

    pub fn handle_road(&mut self, road: &Road, rating: Rating) {
        // Find all matching boundaries
        for obj in self
            .rtree
            .locate_in_envelope_intersecting(&road.geom.envelope())
        {
            // TODO Or even just intersects, to handle boundaries?
            if obj.geom().contains(&road.geom) {
                let info = self.info.get_mut(&obj.data).unwrap();
                info.counts[rating] += 1;
                info.total_length[rating] += road.length;
            }
        }
    }

    /// Write all boundaries with non-zero counts
    pub fn write_output(mut self, path: &str) -> Result<()> {
        let mut summaries_writer = FeatureWriter::from_writer(BufWriter::new(File::create(path)?));
        for obj in self.rtree.drain() {
            let info = &self.info[&obj.data];
            if info.counts[Rating::Red] + info.counts[Rating::Amber] + info.counts[Rating::Green]
                == 0
            {
                continue;
            }

            let mut f = Feature::from(Value::from(obj.geom()));
            f.set_property("name", obj.data);
            f.set_property("red_count", info.counts[Rating::Red]);
            f.set_property("amber_count", info.counts[Rating::Amber]);
            f.set_property("green_count", info.counts[Rating::Green]);
            f.set_property("red_length", info.total_length[Rating::Red]);
            f.set_property("amber_length", info.total_length[Rating::Amber]);
            f.set_property("green_length", info.total_length[Rating::Green]);
            summaries_writer.write_feature(&f)?;
        }
        Ok(())
    }
}
