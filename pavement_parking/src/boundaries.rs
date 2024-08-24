use std::collections::HashMap;
use std::io::BufWriter;

use anyhow::Result;
use enum_map::EnumMap;
use fs_err::File;
use geo::{Contains, MultiPolygon, Polygon};
use geojson::{Feature, FeatureCollection, FeatureWriter, Value};
use rstar::{primitives::GeomWithData, RTree, RTreeObject};

use crate::{Intervention, Rating, Road, Scenario};

/// Aggregate counts per LAD and CA boundaries
pub struct Boundaries {
    rtree: RTree<GeomWithData<Polygon, String>>,
    info: HashMap<String, Boundary>,
}

struct Boundary {
    any_matches: bool,
    // Given a scenario and rating, count the roads and sum the length
    counts: EnumMap<Scenario, EnumMap<Rating, usize>>,
    total_length: EnumMap<Scenario, EnumMap<Rating, f64>>,

    intervention_counts: EnumMap<Intervention, usize>,
    intervention_total_length: EnumMap<Intervention, f64>,
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
                    any_matches: false,
                    counts: EnumMap::default(),
                    total_length: EnumMap::default(),
                    intervention_counts: EnumMap::default(),
                    intervention_total_length: EnumMap::default(),
                },
            );
        }
        println!("Building RTree of {} boundaries", boundaries.len());
        let rtree = RTree::bulk_load(boundaries);

        println!("");
        Ok(Self { rtree, info })
    }

    pub fn handle_road(&mut self, road: &Road) {
        // Find all matching boundaries
        for obj in self
            .rtree
            .locate_in_envelope_intersecting(&road.geom.envelope())
        {
            // TODO Or even just intersects, to handle boundaries?
            if obj.geom().contains(&road.geom) {
                let info = self.info.get_mut(&obj.data).unwrap();
                info.any_matches = true;
                for (scenario, counts) in &mut info.counts {
                    counts[road.ratings[scenario]] += 1;
                }
                for (scenario, lengths) in &mut info.total_length {
                    lengths[road.ratings[scenario]] += road.length;
                }
                info.intervention_counts[road.intervention] += 1;
                info.intervention_total_length[road.intervention] += road.length;
            }
        }
    }

    /// Write all boundaries with at least one road
    pub fn write_output(mut self, path: &str) -> Result<()> {
        let mut summaries_writer = FeatureWriter::from_writer(BufWriter::new(File::create(path)?));
        for obj in self.rtree.drain() {
            let info = &self.info[&obj.data];
            if !info.any_matches {
                continue;
            }

            let mut f = Feature::from(Value::from(obj.geom()));
            f.set_property("name", obj.data);

            // Flatten the table as string keys, since maplibre / MVT doesn't work well with nested
            // properties
            for (scenario, counts) in &info.counts {
                for (rating, count) in counts {
                    f.set_property(format!("counts_{:?}_{}", scenario, rating.to_str()), *count);
                }
            }
            for (scenario, lengths) in &info.total_length {
                for (rating, length) in lengths {
                    f.set_property(
                        format!("lengths_{:?}_{}", scenario, rating.to_str()),
                        length.round() as usize,
                    );
                }
            }
            for (intervention, count) in &info.intervention_counts {
                f.set_property(format!("intervention_counts_{:?}", intervention), *count);
            }
            for (intervention, length) in &info.intervention_total_length {
                f.set_property(
                    format!("intervention_lengths_{:?}", intervention),
                    length.round() as usize,
                );
            }

            summaries_writer.write_feature(&f)?;
        }
        Ok(())
    }
}
