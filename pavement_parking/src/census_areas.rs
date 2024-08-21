use std::collections::HashMap;
use std::io::BufWriter;

use anyhow::{bail, Result};
use fs_err::File;
use gdal::vector::LayerAccess;
use gdal::Dataset;
use geo::{GeodesicLength, Geometry, Intersects, LineString, MultiPolygon, Polygon};
use geojson::{Feature, FeatureWriter, Value};
use indicatif::{ProgressBar, ProgressStyle};
use rstar::{primitives::GeomWithData, RTree, RTreeObject};

use crate::{Class, Rating};

pub struct CensusAreas {
    rtree: RTree<GeomWithData<Polygon, String>>,
    info: HashMap<String, CensusArea>,
}

struct CensusArea {
    number_of_cars_and_vans: f64,
    aggregate_kerb_length: f64,
}

impl CensusAreas {
    pub fn load(path: &str) -> Result<Self> {
        println!("Reading census areas from {path}");

        let dataset = Dataset::open(path)?;
        // Assume only one layer
        let mut layer = dataset.layer(0)?;

        let progress = ProgressBar::new(layer.feature_count()).with_style(ProgressStyle::with_template(
            "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());
        let mut output_areas = Vec::new();
        let mut info = HashMap::new();

        for input_feature in layer.features() {
            progress.inc(1);
            let geo = input_feature.geometry().unwrap().to_geo()?;
            let mp = match geo {
                Geometry::Polygon(p) => MultiPolygon(vec![p]),
                Geometry::MultiPolygon(mp) => mp,
                _ => bail!("Unexpected geometry type"),
            };
            let Some(name) = input_feature.field_as_string_by_name("GEO_ID")? else {
                bail!("Missing GEO_ID");
            };
            let Some(number_of_cars_and_vans) = input_feature
                .field_as_double_by_name("Number of cars or vans: Total: All households")?
            else {
                bail!("Missing number_of_cars_and_vans");
            };

            // MultiPolygon isn't supported, so just insert multiple names
            for polygon in mp {
                output_areas.push(GeomWithData::new(polygon, name.clone()));
            }

            info.insert(
                name,
                CensusArea {
                    number_of_cars_and_vans,
                    aggregate_kerb_length: 0.0,
                },
            );
        }
        println!("Building RTree of {} output areas", output_areas.len());
        let rtree = RTree::bulk_load(output_areas);

        println!("");
        Ok(Self { rtree, info })
    }

    // Returns the GEOID of the census area the road is assigned to, and the parkable length. Also
    // updates the census_areas with the kerb length per assigned road segment
    pub fn aggregate_kerb_length_per_oa(
        &mut self,
        geom: &LineString,
        average_rating: Rating,
        class: Class,
    ) -> Result<(Option<String>, f64)> {
        // For each output area, sum the kerb length where it is possible to park a car.
        // Calculate the parkable kerb length per car in the area.

        // Estimate the length of the kerb where it is possible to park a car
        let parkable_length = parkable_kerb_length(geom, average_rating, class);

        // Assign each road to exactly one output area. If it intersects multiple output areas,
        // it can be assigned by arbitrary but repeatable method.
        // For reproducibility, find all of the of the output areas which intersect the road segment
        // Then select the one with the alphabetically first GEOID.
        let target_geoid = self
            .rtree
            .locate_in_envelope_intersecting_mut(&geom.envelope())
            .filter(|oa| oa.geom().intersects(geom))
            .map(|oa| oa.data.clone())
            .min();
        if let Some(ref geoid) = target_geoid {
            self.info.get_mut(geoid).unwrap().aggregate_kerb_length += parkable_length;
        }

        Ok((target_geoid, parkable_length))
    }

    /// Write all output_areas with non-zero kerb length
    pub fn write_output(mut self, path: &str) -> Result<()> {
        let mut writer = FeatureWriter::from_writer(BufWriter::new(File::create(path)?));
        for obj in self.rtree.drain() {
            let info = &self.info[&obj.data];
            if info.aggregate_kerb_length > 0.0 {
                let mut f = Feature::from(Value::from(obj.geom()));
                f.set_property("GEO_ID", obj.data);
                f.set_property("number_of_cars_and_vans", info.number_of_cars_and_vans);
                f.set_property("aggregate_kerb_length", info.aggregate_kerb_length);
                f.set_property(
                    "kerb_length_per_car",
                    info.aggregate_kerb_length / info.number_of_cars_and_vans,
                );
                writer.write_feature(&f)?;
            }
        }
        Ok(())
    }
}

fn parkable_kerb_length(geom: &LineString, rating: Rating, class: Class) -> f64 {
    // Returns the length of the kerb where it is possible to park a car
    // i.e. not on a junction or a pedestrian crossing, etc.
    // This attempts to implement the table of proposed interventions in
    // the Pavement Parking Assessment Document

    // TODO Haversine?
    let raw_length = geom.geodesic_length();

    let kerb_length = match rating {
        // If the road is wide enough, assume that both sides are parkable
        Rating::Green => 2.0 * raw_length,
        Rating::Amber => raw_length,
        Rating::Red => match class {
            Class::A => 0.0,
            Class::B | Class::C | Class::Unclassified => raw_length,
        },
    };

    // TODO - additional considerations:
    // - One-way roads. Is the widths/rating relationship the same as for two-way roads?
    // - Roads with parking restrictions (including residence parking). How to handle these?
    // - Maybe subtract a length near each junction, pedestrian crossing, school entrance, etc.
    // - For short roads, would the same intervention be applied as for long roads?

    kerb_length
}
