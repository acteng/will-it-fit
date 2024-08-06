use std::io::BufWriter;

use anyhow::{bail, Result};
use fs_err::File;
use gdal::vector::LayerAccess;
use gdal::Dataset;
use geo::{Coord, MapCoordsInPlace};
use geojson::FeatureWriter;
use indicatif::{ProgressBar, ProgressStyle};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Call with path to trn_ntwk_roadlink.gpkg");
        std::process::exit(1);
    }

    gpkg_to_geojson(&args[1], "out.geojson", get_properties)
}

fn gpkg_to_geojson<F: Fn(&gdal::vector::Feature, &mut geojson::Feature) -> Result<bool>>(
    input_path: &str,
    output_path: &str,
    extract_properties: F,
) -> Result<()> {
    let dataset = Dataset::open(input_path)?;
    // Assume only one layer
    let mut layer = dataset.layer(0)?;

    let progress = ProgressBar::new(layer.feature_count()).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());

    let mut writer = FeatureWriter::from_writer(BufWriter::new(File::create(output_path)?));

    let mut count = 0;
    for input_feature in layer.features() {
        progress.inc(1);
        let mut geo = input_feature.geometry().unwrap().to_geo()?;
        // Remove unnecessary precision
        geo.map_coords_in_place(|Coord { x, y }| Coord {
            x: trim_f64(x),
            y: trim_f64(y),
        });

        let mut output_feature = geojson::Feature::from(geojson::Value::from(&geo));
        if extract_properties(&input_feature, &mut output_feature)? {
            writer.write_feature(&output_feature)?;
        }
    }

    println!("Wrote {output_path}");
    Ok(())
}

fn trim_f64(x: f64) -> f64 {
    (x * 10e6).round() / 10e6
}

fn get_properties(input: &gdal::vector::Feature, output: &mut geojson::Feature) -> Result<bool> {
    let Some(average) = input.field_as_double_by_name("roadwidth_average")? else {
        return Ok(false);
    };
    let Some(minimum) = input.field_as_double_by_name("roadwidth_minimum")? else {
        return Ok(false);
    };
    let Some(class) = input.field_as_string_by_name("roadclassification")? else {
        return Ok(false);
    };

    // Skip roads that shouldn't be analyzed for pavement parking
    if class == "Motorway" {
        return Ok(false);
    }

    output.set_property("average_width", average);
    output.set_property("minimum_width", minimum);
    output.set_property("average_rating", rating(&class, average)?);
    output.set_property("minimum_rating", rating(&class, minimum)?);
    output.set_property("class", class);

    Ok(true)
}

fn rating(class: &str, width: f64) -> Result<&'static str> {
    // See https://www.ordnancesurvey.co.uk/documents/os-open-roads-user-guide.pdf page 22 for the
    // cases. The width thresholds come from a TBD table.
    match class {
        "A Road" | "B Road" => Ok(if width >= 11.8 {
            "green"
        } else if width >= 10.4 {
            "amber"
        } else {
            "red"
        }),

        // Note "Classified Unnumbered" is how OS calls C Roads
        "Classified Unnumbered" | "Unclassified" => Ok(if width >= 9.0 {
            "green"
        } else if width >= 7.5 {
            "amber"
        } else {
            // TODO Table doesn't handle [7, 7.5]
            "red"
        }),

        // TODO Need to see what these are
        "Unknown" | "Not Classified" => Ok("TODO"),

        _ => bail!("Unknown roadclassification {class}"),
    }
}
