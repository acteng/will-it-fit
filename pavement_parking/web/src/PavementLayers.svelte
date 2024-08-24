<script lang="ts">
  import { constructMatchExpression } from "svelte-utils/map";
  import {
    LineLayer,
    Popup,
    VectorTileSource,
    hoverStateFilter,
  } from "svelte-maplibre";
  import {
    colors,
    scenarios,
    interventions,
    type Mode,
    type Filters,
  } from "./types";
  import type { ExpressionSpecification } from "maplibre-gl";

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
    <Popup openOn="hover" let:data>
      {#if data?.properties}
        <h1>{data.properties.name || "Unnamed road"}</h1>
        <p>Class: {data.properties.class} road</p>
        <p>Direction: {data.properties.direction}</p>
        <p>Length: {data.properties.length} meters</p>

        <hr />

        <p>
          Carriageway width: {data.properties.road_average_width} average, {data
            .properties.road_minimum_width} minimum
        </p>
        <p>Pavement average width: {data.properties.pavement_average_width}</p>

        <hr />

        {#each scenarios as [scenario, label]}
          <p>
            In scenario {scenario} ({label}), rating is {data.properties[
              `rating_${scenario}`
            ]}
          </p>
        {/each}
        <p>
          Intervention required: {interventions[data.properties.intervention]}
        </p>
      {/if}
    </Popup>
  </LineLayer>
</VectorTileSource>
