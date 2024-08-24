use anyhow::{bail, Result};
use enum_map::EnumMap;
use geo::{Coord, HaversineLength, LineString, MapCoordsInPlace};
use geojson::{Feature, Value};

use crate::{Intervention, Rating, Scenario};

// All distance units are in meters
pub struct Road {
    pub geom: LineString,
    pub length: f64,

    pub name: String,
    pub class: Class,
    pub direction: String,

    pub road_average_width: f64,
    pub road_minimum_width: f64,
    /// Assume that where there are pavements on both sides of the road, then this value is the sum
    /// of both pavements. If there is only one pavement, then this value is the width of that.
    pub pavement_average_width: f64,

    pub ratings: EnumMap<Scenario, Rating>,
    pub intervention: Intervention,
}

#[derive(Clone, Copy, Debug)]
pub enum Class {
    A,
    B,
    C,
    Unclassified,
}

impl Road {
    /// Parse data about one road from the input gpkg. `None` means to skip this road.
    pub fn new(input: gdal::vector::Feature) -> Result<Option<Self>> {
        let Some(class) = input.field_as_string_by_name("roadclassification")? else {
            bail!("Missing roadclassification");
        };
        let class = match class.as_str() {
            "A Road" => Class::A,
            "B Road" => Class::B,
            "Classified Unnumbered" => Class::C,
            "Unclassified" => Class::Unclassified,

            // Skip roads that shouldn't be analyzed for pavement parking
            "Motorway" | "Unknown" | "Not Classified" => {
                return Ok(None);
            }
            _ => bail!("Unknown roadclassification {class}"),
        };

        let mut geom: LineString = input.geometry().unwrap().to_geo()?.try_into()?;
        // Remove unnecessary precision
        geom.map_coords_in_place(|Coord { x, y }| Coord {
            x: trim_wgs84(x),
            y: trim_wgs84(y),
        });
        let length = geom.haversine_length();

        let Some(road_average_width) = input.field_as_double_by_name("roadwidth_average")? else {
            // Sometimes this really is missing
            return Ok(None);
        };
        let Some(road_minimum_width) = input.field_as_double_by_name("roadwidth_minimum")? else {
            // Sometimes this really is missing
            return Ok(None);
        };

        let Some(pavement_average_width) =
            input.field_as_double_by_name("presenceofpavement_averagewidth_m")?
        else {
            bail!("Missing presenceofpavement_averagewidth_m");
        };

        let direction = match input
            .field_as_string_by_name("directionality")?
            .unwrap()
            .as_str()
        {
            "Both Directions" => "both".to_string(),
            "In Direction" | "In Opposite Direction" => "one-way".to_string(),
            x => bail!("Unknown directionality {x}"),
        };

        let name = input
            .field_as_string_by_name("name1_text")?
            .unwrap_or_else(String::new);

        // TODO Only consider road width as input, or do we want to continue to also try with
        // pavement width?
        let ratings = EnumMap::from_fn(|scenario| Rating::new(scenario, class, road_average_width));
        let intervention = Intervention::calculate(&ratings, direction == "one-way");

        Ok(Some(Self {
            geom,
            length,

            name,
            class,
            direction,

            road_average_width,
            road_minimum_width,
            pavement_average_width,

            ratings,
            intervention,
        }))
    }

    pub fn to_gj(self, parkable_length: f64, output_area_geoid: Option<String>) -> Feature {
        let mut f = Feature::from(Value::from(&self.geom));
        f.set_property("length", trim_meters(self.length));

        f.set_property("name", self.name);
        f.set_property("class", format!("{:?}", self.class));
        f.set_property("direction", self.direction);

        f.set_property("road_average_width", self.road_average_width);
        f.set_property("road_minimum_width", self.road_minimum_width);
        f.set_property("pavement_average_width", self.pavement_average_width);

        for (scenario, rating) in self.ratings {
            f.set_property(format!("rating_{:?}", scenario), rating.to_str());
        }
        f.set_property("intervention", format!("{:?}", self.intervention));

        f.set_property("parkable_length", trim_meters(parkable_length));
        // TODO Just debug right now, not used in the UI
        f.set_property(
            "output_area_geoid",
            output_area_geoid.unwrap_or("NONE".to_string()),
        );

        f
    }
}

fn trim_wgs84(x: f64) -> f64 {
    (x * 10e6).round() / 10e6
}

fn trim_meters(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}
