use std::collections::HashMap;
use std::io::BufWriter;

use anyhow::{bail, Result};
use fs_err::File;
use gdal::vector::LayerAccess;
use gdal::Dataset;
use geo::{Contains, Coord, LineString, MapCoordsInPlace, MultiPolygon, Polygon};
use geojson::{FeatureCollection, FeatureWriter};
use indicatif::{ProgressBar, ProgressStyle};
use rstar::{primitives::GeomWithData, RTree, RTreeObject};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Call with path to trn_ntwk_roadlink.gpkg");
        std::process::exit(1);
    }

    let boundaries = read_boundaries("inputs/boundaries.geojson")?;
    gpkg_to_geojson(&args[1], "web/public/out.geojson", process_feature, boundaries)
}

fn gpkg_to_geojson<
    F: Fn(
        LineString,
        gdal::vector::Feature,
        &mut Boundaries,
        &mut FeatureWriter<BufWriter<File>>,
    ) -> Result<()>,
>(
    input_path: &str,
    output_path: &str,
    process: F,
    mut boundaries: Boundaries,
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
        let ls: LineString = geo.try_into()?;

        process(ls, input_feature, &mut boundaries, &mut writer)?;
    }

    // Write all boundaries with non-zero counts
    for obj in boundaries.rtree.drain() {
        let mut f = geojson::Feature::from(geojson::Value::from(obj.geom()));
        let counts = boundaries.counts[&obj.data];
        if counts[0] + counts[1] + counts[2] > 0 {
            f.set_property("name", obj.data);
            f.set_property("red", counts[0]);
            f.set_property("amber", counts[1]);
            f.set_property("green", counts[2]);
            writer.write_feature(&f)?;
        }
    }

    println!("Wrote {output_path}");
    Ok(())
}

fn trim_f64(x: f64) -> f64 {
    (x * 10e6).round() / 10e6
}

fn process_feature(
    geom: LineString,
    input: gdal::vector::Feature,
    boundaries: &mut Boundaries,
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

    let average_rating = rating(&class, average)?;
    let minimum_rating = rating(&class, minimum)?;

    // Find all matching boundaries
    for obj in boundaries
        .rtree
        .locate_in_envelope_intersecting(&geom.envelope())
    {
        // TODO Or even just intersects, to handle boundaries?
        if obj.geom().contains(&geom) {
            let count = boundaries.counts.get_mut(&obj.data).unwrap();
            // TODO Use average_rating for now
            if average_rating == "red" {
                count[0] += 1;
            } else if average_rating == "amber" {
                count[1] += 1;
            } else {
                count[2] += 1;
            }
        }
    }

    // Include the road in the output
    let mut output_line = geojson::Feature::from(geojson::Value::from(&geom));
    output_line.set_property("average_width", average);
    output_line.set_property("minimum_width", minimum);
    output_line.set_property("average_rating", average_rating);
    output_line.set_property("minimum_rating", minimum_rating);
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

struct Boundaries {
    rtree: RTree<GeomWithData<Polygon, String>>,
    // Per boundary name, the count for [red, amber, green]
    counts: HashMap<String, [usize; 3]>,
}

fn read_boundaries(path: &str) -> Result<Boundaries> {
    let gj: FeatureCollection = fs_err::read_to_string(path)?.parse()?;
    let mut boundaries = Vec::new();
    let mut counts = HashMap::new();
    for f in gj.features {
        let name = f.property("name").unwrap().as_str().unwrap().to_string();
        let mp: MultiPolygon = if matches!(
            f.geometry.as_ref().unwrap().value,
            geojson::Value::Polygon(_)
        ) {
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
    let rtree = RTree::bulk_load(boundaries);

    Ok(Boundaries { rtree, counts })
}
