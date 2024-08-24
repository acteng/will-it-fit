use std::io::BufWriter;
use std::process::Command;

use anyhow::{bail, Result};
use fs_err::File;
use gdal::vector::LayerAccess;
use gdal::Dataset;
use geojson::FeatureWriter;
use indicatif::{ProgressBar, ProgressStyle};

use crate::boundaries::Boundaries;
use crate::census_areas::CensusAreas;
use crate::ratings::{Rating, Scenario};
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

    tippecanoe(
        &pavements_output_path,
        &pavements_output_path.replace("geojson", "pmtiles"),
    )?;
    tippecanoe(
        &output_areas_output_path,
        &output_areas_output_path.replace("geojson", "pmtiles"),
    )?;

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

    boundaries.handle_road(&road);

    let (output_area_geoid, parkable_length) = census_areas.aggregate_kerb_length_per_oa(&road)?;

    writer.write_feature(&road.to_gj(parkable_length, output_area_geoid))?;

    Ok(())
}

fn tippecanoe(input: &str, output: &str) -> Result<()> {
    let mut cmd = Command::new("tippecanoe");
    cmd.arg(input)
        .arg("-o")
        .arg(output)
        .arg("--force") // Overwrite existing output
        .arg("--generate-ids")
        .arg("-l")
        .arg("pavements")
        .arg("-Z10")
        .arg("-z11")
        .arg("--drop-densest-as-needed");
    println!("Running: {cmd:?}");
    if !cmd.status()?.success() {
        bail!("tippecanoe failed");
    }
    Ok(())
}
