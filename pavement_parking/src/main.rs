use std::any::Any;
use std::hash::Hash;
use std::{collections::HashMap, os::macos::raw};
use std::io::BufWriter;

use anyhow::{bail, Result};
use fs_err::File;
use gdal::vector::LayerAccess;
use gdal::Dataset;
use geo::{Contains, Coord, GeodesicLength, Intersects, LineString, MapCoordsInPlace, MultiPolygon, Polygon};
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
    let census_areas = read_census_areas("../data_prep/car_ownership/car_ownership.geojson")?;
    gpkg_to_geojson(
        &args[1],
        "web/public/summaries.geojson",
        "web/public/pavements.geojson",
        "web/public/output_areas.geojson",
        process_feature,
        boundaries,
        census_areas,
    )
}

fn gpkg_to_geojson<
    F: Fn(
        LineString,
        gdal::vector::Feature,
        &mut Boundaries,
        &mut CensusAreas,
        &mut FeatureWriter<BufWriter<File>>,
    ) -> Result<()>,
>(
    input_path: &str,
    summaries_output_path: &str,
    pavements_output_path: &str,
    output_areas_output_path: &str,
    process: F,
    mut boundaries: Boundaries,
    mut census_areas: CensusAreas,
) -> Result<()> {
    let dataset = Dataset::open(input_path)?;
    // Assume only one layer
    let mut layer = dataset.layer(0)?;

    let progress = ProgressBar::new(layer.feature_count()).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());

    let mut pavements_writer =
        FeatureWriter::from_writer(BufWriter::new(File::create(pavements_output_path)?));

    for input_feature in layer.features() {
        progress.inc(1);
        let mut geo = input_feature.geometry().unwrap().to_geo()?;
        // Remove unnecessary precision
        geo.map_coords_in_place(|Coord { x, y }| Coord {
            x: trim_f64(x),
            y: trim_f64(y),
        });
        let ls: LineString = geo.try_into()?;

        process(ls, input_feature, &mut boundaries, &mut census_areas, &mut pavements_writer)?;
    }

    // Write all boundaries with non-zero counts
    let mut summaries_writer =
        FeatureWriter::from_writer(BufWriter::new(File::create(summaries_output_path)?));
    for obj in boundaries.rtree.drain() {
        let mut f = geojson::Feature::from(geojson::Value::from(obj.geom()));
        let counts = boundaries.counts[&obj.data];
        if counts[0] + counts[1] + counts[2] > 0 {
            f.set_property("name", obj.data);
            f.set_property("red", counts[0]);
            f.set_property("amber", counts[1]);
            f.set_property("green", counts[2]);
            summaries_writer.write_feature(&f)?;
        }
    }

    // Write all output_areas with non-zero counts
    let mut oa_writer =
        FeatureWriter::from_writer(BufWriter::new(File::create(output_areas_output_path)?));
    for obj in census_areas.rtree.drain() {
        let mut f = geojson::Feature::from(geojson::Value::from(obj.geom()));
        let params = &census_areas.params[&obj.data];

        if params["aggregate_kerb_length"].is_some() {
            f.set_property("GEO_ID", obj.data);
            f.set_property("number_of_cars_and_vans", params["number_of_cars_and_vans"].unwrap());
            f.set_property("aggregate_kerb_length", params["aggregate_kerb_length"].unwrap());
            f.set_property(
                "kerb_length_per_car",
                params["aggregate_kerb_length"].unwrap() / params["number_of_cars_and_vans"].unwrap()
            );
            oa_writer.write_feature(&f)?;
        }
    }

    println!("Wrote '{summaries_output_path}', '{pavements_output_path}' and '{output_areas_output_path}'");
    Ok(())
}

fn trim_f64(x: f64) -> f64 {
    (x * 10e6).round() / 10e6
}

fn process_feature(
    geom: LineString,
    input: gdal::vector::Feature,
    boundaries: &mut Boundaries,
    census_areas: &mut CensusAreas,
    writer: &mut FeatureWriter<BufWriter<File>>,
) -> Result<()> {
    let Some(road_average) = input.field_as_double_by_name("roadwidth_average")? else {
        return Ok(());
    };
    let Some(road_minimum) = input.field_as_double_by_name("roadwidth_minimum")? else {
        return Ok(());
    };
    let Some(class) = input.field_as_string_by_name("roadclassification")? else {
        return Ok(());
    };

    // Assume that where there are pavements on both sides of the road, then this value is the
    // sum of both pavements. If there is only one pavement, then this value is the width of that.
    let Some(pavement_average) =
        input.field_as_double_by_name("presenceofpavement_averagewidth_m")?
    else {
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

    let average_rating_inc_pavements = rating(&class, road_average + pavement_average)?;
    let average_rating_exc_pavements = rating(&class, road_average)?;
    let minimum_rating = rating(&class, road_minimum)?;

    let rating_change = if average_rating_inc_pavements == average_rating_exc_pavements {
        "no_change"
    } else {
        average_rating_exc_pavements
    };

    let (output_area_geoid, parkable_length ) = aggregate_kerb_length_per_oa(
        census_areas, &geom, average_rating, &class)?;

    // Find all matching boundaries
    for obj in boundaries
        .rtree
        .locate_in_envelope_intersecting(&geom.envelope())
    {
        // TODO Or even just intersects, to handle boundaries?
        if obj.geom().contains(&geom) {
            let count = boundaries.counts.get_mut(&obj.data).unwrap();
            // TODO Use average_rating_exc_pavements for now
            if average_rating_exc_pavements == "red" {
                count[0] += 1;
            } else if average_rating_exc_pavements == "amber" {
                count[1] += 1;
            } else if average_rating_exc_pavements == "green" {
                count[2] += 1;
            } else {
                // No change in rating
                count[3] += 1;
            }
        }
    }

    // Include the road in the output
    let mut output_line = geojson::Feature::from(geojson::Value::from(&geom));
    output_line.set_property("average_width", road_average);
    output_line.set_property("minimum_width", road_minimum);
    output_line.set_property("pavement_average_width", pavement_average_width);
    output_line.set_property("average_rating", average_rating_exc_pavements);
    output_line.set_property("average_rating_inc_pavements", average_rating_inc_pavements);
    output_line.set_property("minimum_rating", minimum_rating);
    output_line.set_property("rating_change", rating_change);
    output_line.set_property("parkable_length", parkable_length);
    output_line.set_property("output_area_geoid", output_area_geoid.as_str());
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

fn parkable_kerb_length(geom: &LineString, rating: &str, class: &str) -> f64 {
    // Returns the length of the kerb where it is possible to park a car
    // i.e. not on a junction or a pedestrian crossing, etc.
    // This attempts to implement the table of proposed interventions in
    // the Pavement Parking Assessment Document

    let raw_length = geom.geodesic_length();

    let kerb_length = match rating{
        // If the road is wide enough, assume that both sides are parkable
        "green" => 2.0 * raw_length,
        "amber" => raw_length,
        "red" => match class {
            "A Road" => 0.0,
            "B Road" | "Classified Unnumbered" | "Unclassified" => raw_length,
            "Unknown" | "Not Classified" => raw_length,
            _ => panic!("Unknown class {class}"),
            
        },
        "TODO" => raw_length,
        _ => panic!("Unknown rating {rating}"),
    };

    // TODO - additional considerations:
    // - One-way roads. Is the widths/rating relationship the same as for two-way roads?
    // - Roads with parking restrictions (including residence parking). How to handle these?
    // - Maybe subtract a length near each junction, pedestrian crossing, school entrance, etc.
    // - For short roads, would the same intervention be applied as for long roads?

    kerb_length
}

fn aggregate_kerb_length_per_oa(
    census_area: &mut CensusAreas,
    geom: &LineString,
    average_rating: &str,
    road_class: &str,
) -> Result<(String, f64)> {
    // For each output area, sum the kerb length where it is possible to park a car.
    // Calculate the parkable kerb length per car in the area.
    // Returns the GEOID of the census area each road is assigned to
    // Updates the census_areas with the kerb length per assigned road segment 

    // Estimate the length of the kerb where it is possible to park a car
    let parkable_length = parkable_kerb_length(geom, average_rating, road_class);

    // Assign each road to exactly one output area. If it intersects multiple output areas,
    // it can be assigned by arbitrary but repeatable method.
    // For reproducibility, find all of the of the output areas which intersect the road segment
    // Then select the one with the alphabetically first GEOID.
    let mut target_geoid = None::<&String>;
    for oa in census_area
        .rtree
        .locate_in_envelope_intersecting_mut(&geom.envelope())
    {
        if oa.geom().intersects(geom) {
            let next_geoid = &oa.data;
            target_geoid = match target_geoid {
                None => Some(next_geoid),
                Some(ref current) => {
                    if next_geoid < *current {
                        Some(next_geoid)
                    } else {
                        target_geoid
                    }
                }
            }
        }
    }

    if target_geoid.is_some() {
        let sub_params = census_area.params.get_mut(target_geoid.unwrap()).unwrap();
    
        sub_params.insert(
            "aggregate_kerb_length".to_string(),
            Some(sub_params["aggregate_kerb_length"].unwrap_or(0.0) + parkable_length),
        );
    }

    Ok((target_geoid.unwrap_or(&String::from("NONE")).clone(), parkable_length))
}


struct Boundaries {
    rtree: RTree<GeomWithData<Polygon, String>>,
    // Per boundary name, the count for [red, amber, green]
    counts: HashMap<String, [usize; 4]>
}

struct CensusAreas {
    rtree: RTree<GeomWithData<Polygon, String>>,
    // Per boundary name, the count for [red, amber, green]
    params: HashMap<String, HashMap<String, Option<f64>>>
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
        counts.insert(name, [0, 0, 0, 0]);
    }
    let rtree = RTree::bulk_load(boundaries);

    Ok(Boundaries { rtree, counts})
}


fn read_census_areas(path: &str) -> Result<CensusAreas> {
    // Read the census areas and their populations

    let gj: FeatureCollection = fs_err::read_to_string(path)?.parse()?;
    let mut output_areas = Vec::new();
    let mut params = HashMap::new();
    for f in gj.features {
        // dbg!(&f);
        let name = f.property("GEO_ID").unwrap().as_str().unwrap().to_string();
        let number_of_cars_and_vans = f.property("number_of_cars_and_vans").unwrap().as_f64();

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
            output_areas.push(GeomWithData::new(polygon, name.clone()));
        }
        
        let sub_params = HashMap::from([
            ("number_of_cars_and_vans".to_string(), number_of_cars_and_vans),
            ("aggregate_kerb_length".to_string(), None::<f64>),
            // ("kerb_length_per_car".to_string(), None::<f64>),
        ]);

        params.insert(name, sub_params);
    }
    let rtree = RTree::bulk_load(output_areas);

    Ok(CensusAreas { rtree, params})
}

