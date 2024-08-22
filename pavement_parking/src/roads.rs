use anyhow::{bail, Result};
use geo::{Coord, LineString, MapCoordsInPlace};

pub struct Road {
    pub geom: LineString,
    pub class: Class,
    pub road_average_width: f64,
    pub road_minimum_width: f64,
    /// Assume that where there are pavements on both sides of the road, then this value is the sum
    /// of both pavements. If there is only one pavement, then this value is the width of that.
    pub pavement_average_width: f64,
    pub direction: String,
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
            x: trim_f64(x),
            y: trim_f64(y),
        });

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

        Ok(Some(Self {
            geom,
            class,
            road_average_width,
            road_minimum_width,
            pavement_average_width,
            direction,
        }))
    }
}

fn trim_f64(x: f64) -> f64 {
    (x * 10e6).round() / 10e6
}
