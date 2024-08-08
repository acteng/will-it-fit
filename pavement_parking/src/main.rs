use std::io::BufWriter;

use anyhow::{bail, Result};
use fs_err::File;
use gdal::vector::LayerAccess;
use gdal::Dataset;
use geo::{Coord, Geometry, MapCoordsInPlace, MultiPolygon};
use geojson::{FeatureCollection, FeatureWriter};
use indicatif::{ProgressBar, ProgressStyle};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Call with path to trn_ntwk_roadlink.gpkg");
        std::process::exit(1);
    }

    let boundaries = read_boundaries("inputs/boundaries.geojson")?;
    gpkg_to_geojson(&args[1], "out.geojson", process_feature)
}

fn gpkg_to_geojson<
    F: Fn(Geometry, gdal::vector::Feature, &mut FeatureWriter<BufWriter<File>>) -> Result<()>,
>(
    input_path: &str,
    output_path: &str,
    process: F,
) -> Result<()> {
    let dataset = Dataset::open(input_path)?;
    // Assume only one layer
    let mut layer = dataset.layer(0)?;

    let progress = ProgressBar::new(layer.feature_count()).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());

    let mut writer = FeatureWriter::from_writer(BufWriter::new(File::create(output_path)?));

    for input_feature in layer.features() {
        progress.inc(1);
        let mut geo = input_feature.geometry().unwrap().to_geo()?;
        // Remove unnecessary precision
        geo.map_coords_in_place(|Coord { x, y }| Coord {
            x: trim_f64(x),
            y: trim_f64(y),
        });

        process(geo, input_feature, &mut writer)?;
    }

    println!("Wrote {output_path}");
    Ok(())
}

fn trim_f64(x: f64) -> f64 {
    (x * 10e6).round() / 10e6
}

fn process_feature(
    geom: Geometry,
    input: gdal::vector::Feature,
    writer: &mut FeatureWriter<BufWriter<File>>,
) -> Result<()> {
    let Some(average) = input.field_as_double_by_name("roadwidth_average")? else {
        return Ok(());
    };
    let Some(minimum) = input.field_as_double_by_name("roadwidth_minimum")? else {
        return Ok(());
    };
    let Some(class) = input.field_as_string_by_name("roadclassification")? else {
        return Ok(());
    };

    // Skip roads that shouldn't be analyzed for pavement parking
    if class == "Motorway" {
        return Ok(());
    }

    let direction = match input
        .field_as_string_by_name("directionality")?
        .unwrap()
        .as_str()
    {
        "Both Directions" => "both",
        "In Direction" | "In Opposite Direction" => "one-way",
        x => bail!("Unknown directionality {x}"),
    };

    let mut output_line = geojson::Feature::from(geojson::Value::from(&geom));
    output_line.set_property("average_width", average);
    output_line.set_property("minimum_width", minimum);
    output_line.set_property("average_rating", rating(&class, average)?);
    output_line.set_property("minimum_rating", rating(&class, minimum)?);
    output_line.set_property("class", class);
    output_line.set_property("direction", direction);
    writer.write_feature(&output_line)?;

    Ok(())
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

struct Boundary {
    geometry: MultiPolygon,
    name: String,
}

fn read_boundaries(path: &str) -> Result<Vec<Boundary>> {
    let gj: FeatureCollection = fs_err::read_to_string(path)?.parse()?;
    let mut boundaries = Vec::new();
    for f in gj.features {
        let name = f.property("name").unwrap().as_str().unwrap().to_string();
        let geometry: MultiPolygon = if matches!(
            f.geometry.as_ref().unwrap().value,
            geojson::Value::Polygon(_)
        ) {
            MultiPolygon(vec![f.try_into()?])
        } else {
            f.try_into()?
        };
        boundaries.push(Boundary { geometry, name });
    }
    Ok(boundaries)
}
