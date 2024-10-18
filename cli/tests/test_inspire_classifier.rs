use std::collections::HashSet;

mod utils;

use cli::{classify_polygons_by_id, SourceInspirePolygons};

use std::fs::File;
use std::io::BufReader;



struct InspireClassificationExpectedResults {
    case_id: usize,
    road_space_polygons: Option<HashSet<usize>>,
    non_road_space_polygons: Option<HashSet<usize>>,
}

fn get_expected_results() -> Vec<InspireClassificationExpectedResults> {
    // See `test_cases.md`` for details
    vec![
        InspireClassificationExpectedResults {
            case_id: 1,
            road_space_polygons: Some(HashSet::from([
                25855822,
                25852274,
                25853082,
                25857333,
                25853200,
                25853129,
                25853929,
            ])),
            non_road_space_polygons: None,
        },
        InspireClassificationExpectedResults {
            case_id: 3,
            road_space_polygons: Some(HashSet::from([
                25809009,
            ])),
            non_road_space_polygons: Some(HashSet::from([
                25803085,
                25803248,
                54395749,
            ])),
        },
        InspireClassificationExpectedResults {
            case_id: 5,
            road_space_polygons: Some(HashSet::from([
                25826568,
            ])),
            non_road_space_polygons: None
        },
    ]
}


fn read_test_case_source_polygons(case_id: usize) -> Result<Vec<SourceInspirePolygons>, geojson::Error> {
    // let f_name = format!("test_case_{}", case_id);
    let f_path = format!("test_cases/test_case_{}_inspire_parcels.geojson", case_id);
    let f_path = utils::get_test_file_path(f_path).unwrap();
    dbg!("Reading test case source polygons from: {}", f_path.clone());

    let file_reader = BufReader::new(File::open(f_path)?);

    geojson::de::deserialize_feature_collection_to_vec::<SourceInspirePolygons>(file_reader)
}

fn check_expected_results(
    expected_matches: &Option<HashSet<usize>>,
    expected_non_matches: &Option<HashSet<usize>>,
    actual_matches: &Option<HashSet<usize>>,
) {

    // Handle cases where either the expected or actual results are `None`
    match (expected_matches, actual_matches) {
        (None, _) => {
            // Both are `None`, so we're good
        },
        (Some(expected), Some(actual)) => {
            // We want to check that we've identified all the expected road space 
            // polygons, but our 'expected' list might not be exhaustive.
            assert!(
                actual.is_superset(&expected)
            )
        },
        (Some(_), None) => {
            assert!(false, "Expected and actual road space polygons do not match");
        }
        
    } 

    // Check that we have mis-classified any road space polygons
    match (expected_non_matches, actual_matches) {
        (Some(expected), Some(actual)) => {
            assert!(
                actual.is_disjoint(&expected)
            )
        },
        _ => {
            // We're good
        }
    }
    
}


#[test]
fn test_classify_group_a() {

    let expected_results = get_expected_results();

    for case in expected_results.iter() {
        let case_id = case.case_id;
        let expected_road_space_ids = &case.road_space_polygons;
        let expected_non_road_space_ids = &case.non_road_space_polygons;

        let source_polygons = read_test_case_source_polygons(case_id).unwrap();
        let (actual_road_space, actual_non_road_space) = classify_polygons_by_id(source_polygons);

        // Check the road space polygons
        check_expected_results(
            &expected_road_space_ids,
            &expected_non_road_space_ids,
            &actual_road_space
        );

        // Check the non-road space polygons
        check_expected_results(
            &expected_non_road_space_ids,
            &expected_road_space_ids,
            &actual_non_road_space
        );
    }

}