use std::io::BufWriter;

use anyhow::Result;
use fs_err::File;
use gdal::vector::LayerAccess;
use gdal::Dataset;
use geojson::FeatureWriter;
use indicatif::{ProgressBar, ProgressStyle};

use crate::boundaries::Boundaries;
use crate::census_areas::CensusAreas;
use crate::ratings::Rating;
use crate::roads::{Class, Road};

mod boundaries;
mod census_areas;
mod ratings;
mod roads;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Call with path to trn_ntwk_roadlink.gpkg");
        std::process::exit(1);
    }
    let input_path = &args[1];
    let summaries_output_path = "web/public/summaries.geojson";
    let pavements_output_path = "web/public/pavements.geojson";
    let output_areas_output_path = "web/public/output_areas.geojson";

    let mut boundaries = Boundaries::load("inputs/boundaries.geojson")?;
    let mut census_areas = CensusAreas::load("inputs/car_ownership.gpkg")?;

    println!("Reading {input_path}");
    let dataset = Dataset::open(input_path)?;
    // Assume only one layer
    let mut layer = dataset.layer(0)?;

    let progress = ProgressBar::new(layer.feature_count()).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());

    let mut pavements_writer =
        FeatureWriter::from_writer(BufWriter::new(File::create(pavements_output_path)?));

    for input_feature in layer.features() {
        progress.inc(1);
        handle_road(
            input_feature,
            &mut boundaries,
            &mut census_areas,
            &mut pavements_writer,
        )?;
    }

    boundaries.write_output(summaries_output_path)?;
    census_areas.write_output(output_areas_output_path)?;

    println!("\n\nWrote '{summaries_output_path}', '{pavements_output_path}' and '{output_areas_output_path}'");
    Ok(())
}

fn handle_road(
    input: gdal::vector::Feature,
    boundaries: &mut Boundaries,
    census_areas: &mut CensusAreas,
    writer: &mut FeatureWriter<BufWriter<File>>,
) -> Result<()> {
    let Some(road) = Road::new(input)? else {
        return Ok(());
    };

    let rating_inc_pavements = Rating::new(
        road.class,
        road.road_average_width + road.pavement_average_width,
    );
    let rating_exc_pavements = Rating::new(road.class, road.road_average_width);

    let rating_change = if rating_inc_pavements == rating_exc_pavements {
        "no_change"
    } else {
        rating_exc_pavements.to_str()
    };

    let (output_area_geoid, parkable_length) =
        census_areas.aggregate_kerb_length_per_oa(&road.geom, rating_exc_pavements, road.class)?;

    boundaries.handle_road(&road, rating_exc_pavements);

    // Include the road in the output
    let mut output_line = geojson::Feature::from(geojson::Value::from(&road.geom));
    output_line.set_property("road_average_width", road.road_average_width);
    output_line.set_property("road_minimum_width", road.road_minimum_width);
    output_line.set_property("pavement_average_width", road.pavement_average_width);
    output_line.set_property("rating_exc_pavements", rating_exc_pavements.to_str());
    output_line.set_property("rating_inc_pavements", rating_inc_pavements.to_str());
    output_line.set_property("parkable_length", parkable_length);
    output_line.set_property(
        "output_area_geoid",
        output_area_geoid.unwrap_or("NONE".to_string()),
    );
    output_line.set_property("rating_change", rating_change);
    output_line.set_property("class", format!("{:?}", road.class));
    output_line.set_property("direction", road.direction);
    writer.write_feature(&output_line)?;

    Ok(())
}
