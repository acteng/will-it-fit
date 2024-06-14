import buffer from "@turf/buffer";
import lineOffset from "@turf/line-offset";
import type { Feature, LineString, FeatureCollection, Polygon } from "geojson";

export function renderLanes(
  route: Feature<LineString>,
  lanes: string,
): FeatureCollection<Polygon> {
  let results = {
    type: "FeatureCollection" as const,
    features: [],
  };

  // TODO Make | be offset 0
  let widthSum = 0;
  for (let code of lanes) {
    let [color, width] = {
      s: ["grey", 3],
      c: ["green", 4],
      b: ["red", 6],
      d: ["black", 6],
      "|": ["yellow", 1],
    }[code];
    let shifted = lineOffset(route, widthSum + width / 2, { units: "meters" });
    let buffered = buffer(shifted, width / 2, { units: "meters" });
    if (buffered) {
      buffered.properties = { color };
      results.features.push(buffered);
    }
    widthSum += width;
  }
  return results;
}
