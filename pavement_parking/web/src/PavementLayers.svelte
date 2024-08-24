<script lang="ts">
  import { constructMatchExpression } from "svelte-utils/map";
  import {
    LineLayer,
    VectorTileSource,
    hoverStateFilter,
  } from "svelte-maplibre";
  import { colors, interventionColors, type Mode, type Filters } from "./types";
  import type { ExpressionSpecification } from "maplibre-gl";
  import RoadPopup from "./RoadPopup.svelte";

  export let show: Mode;
  export let url: string;
  export let roadFilters: Filters;

  function makeFilter(f: Filters): ExpressionSpecification {
    let ratings = Object.entries(f.showRatings)
      .filter((pair) => pair[1])
      .map((pair) => pair[0]);
    let classes = Object.entries(f.showClasses)
      .filter((pair) => pair[1])
      .map((pair) => pair[0]);
    let directions = Object.entries(f.showDirections)
      .filter((pair) => pair[1])
      .map((pair) => pair[0]);
    return [
      "all",
      ["has", "class"],
      ["in", ["get", "class"], ["literal", classes]],
      ["in", ["get", `rating_${f.scenario}`], ["literal", ratings]],
      ["in", ["get", "direction"], ["literal", directions]],
    ];
  }
</script>

<VectorTileSource url={`pmtiles://${url}`}>
  <LineLayer
    sourceLayer="pavements"
    filter={makeFilter(roadFilters)}
    layout={{ visibility: show == "roads" ? "visible" : "none" }}
    manageHoverState
    paint={{
      "line-width": hoverStateFilter(5, 10),
      "line-color": constructMatchExpression(
        ["get", `rating_${roadFilters.scenario}`],
        colors,
        "black",
      ),
    }}
    beforeId="Road numbers"
  >
    <RoadPopup />
  </LineLayer>

  <LineLayer
    sourceLayer="pavements"
    layout={{ visibility: show == "interventions" ? "visible" : "none" }}
    manageHoverState
    paint={{
      "line-width": hoverStateFilter(5, 10),
      "line-color": constructMatchExpression(
        ["get", "intervention"],
        interventionColors,
        "yellow",
      ),
    }}
    beforeId="Road numbers"
  >
    <RoadPopup />
  </LineLayer>
</VectorTileSource>
