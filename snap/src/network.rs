use std::collections::HashSet;

use anyhow::{bail, Result};
use geo::{Coord, HaversineDistance, LineString, Point};
use geojson::ser::serialize_geometry;
use petgraph::graphmap::UnGraphMap;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Road {
    #[serde(serialize_with = "serialize_geometry")]
    pub geometry: LineString,
    pub min_width: f64,
    pub avg_width: f64,
    pub length: f64,
}

// Assumes input is already a network; endpoints should match up exactly
pub struct Network {
    roads: Vec<Road>,
    // TODO Do we need to store these?
    intersections: HashSet<Intersection>,

    // usize is edge idx
    // TODO
    graph: UnGraphMap<Intersection, usize>,
}

impl Network {
    pub fn new() -> Self {
        Self {
            roads: Vec::new(),
            intersections: HashSet::new(),
            graph: UnGraphMap::new(),
        }
    }

    pub fn add_road(&mut self, road: Road) {
        let road_id = self.roads.len();
        let i1 = hashify_point(*road.geometry.0.first().unwrap());
        let i2 = hashify_point(*road.geometry.0.last().unwrap());
        self.roads.push(road);
        self.intersections.insert(i1);
        self.intersections.insert(i2);
        self.graph.add_edge(i1, i2, road_id);
    }

    pub fn debug_roads(&self) -> Result<String> {
        Ok(geojson::ser::to_feature_collection_string(&self.roads)?.to_string())
    }

    fn closest_intersection(&self, pt: Point) -> Intersection {
        hashify_point(
            self.roads
                .iter()
                .flat_map(|road| {
                    [
                        road.geometry.points().next().unwrap(),
                        road.geometry.points().last().unwrap(),
                    ]
                })
                .min_by_key(|c| (pt.haversine_distance(c) * 10e6) as usize)
                .unwrap()
                .into(),
        )
    }

    // Snaps the route endpoints to the nearest intersection, then finds the shortest distance
    // path. Dead simple start. Later we can look for snapped vs freehand waypoints. No need for
    // perf, because the network within the bbox of the route is going to be tiny.
    pub fn snap_route(&self, route: LineString) -> Result<String> {
        let node1 = self.closest_intersection(route.points().next().unwrap());
        let node2 = self.closest_intersection(route.points().last().unwrap());

        let Some((_, path)) = petgraph::algo::astar(
            &self.graph,
            node1,
            |i| i == node2,
            |(_, _, edge)| self.roads[*edge].length,
            |_| 0.0,
        ) else {
            bail!("No path from {node1:?} to {node2:?}");
        };

        let mut snapped = Vec::new();
        for pair in path.windows(2) {
            snapped.push(self.roads[*self.graph.edge_weight(pair[0], pair[1]).unwrap()].clone());
        }
        Ok(geojson::ser::to_feature_collection_string(&snapped)?.to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Intersection(isize, isize);

fn hashify_point(pt: Coord) -> Intersection {
    Intersection((pt.x * 1_000_000.0) as isize, (pt.y * 1_000_000.0) as isize)
}
