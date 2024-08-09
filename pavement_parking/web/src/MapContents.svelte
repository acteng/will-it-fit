<script lang="ts">
  import { constructMatchExpression } from "svelte-utils/map";
  import {
    LineLayer,
    GeoJSON,
    Popup,
    hoverStateFilter,
    FillLayer,
  } from "svelte-maplibre";
  import { colors, type Filters } from "./types";
  import type { ExpressionSpecification } from "maplibre-gl";

  export let show: "streets" | "lad-summary" | "ca-summary";
  export let streetFilters: Filters;

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
      ["in", ["get", f.useRating], ["literal", ratings]],
      ["in", ["get", "direction"], ["literal", directions]],
    ];
  }
</script>

<GeoJSON data="/out.geojson" generateId>
  <LineLayer
    filter={makeFilter(streetFilters)}
    layout={{ visibility: show == "streets" ? "visible" : "none" }}
    manageHoverState
    paint={{
      "line-width": hoverStateFilter(5, 10),
      "line-color": constructMatchExpression(
        ["get", streetFilters.useRating],
        {
          green: colors.green,
          amber: colors.amber,
          red: colors.red,
          TODO: "black",
        },
        "black",
      ),
    }}
  >
    <Popup openOn="hover" let:data popupClass="popup">
      {#if data?.properties}
        <h1>{data.properties.class} street</h1>
        <p>Direction: {data.properties.direction}</p>
        <p>
          Average width {data.properties.average_width}, rating {data.properties
            .average_rating}
        </p>
        <p>
          Minimum width {data.properties.minimum_width}, rating {data.properties
            .minimum_rating}
        </p>
      {/if}
    </Popup>
  </LineLayer>

  <FillLayer
    filter={[
      "all",
      ["has", "name"],
      ["in", show == "lad-summary" ? "LAD_" : "CA_", ["get", "name"]],
    ]}
    layout={{ visibility: show.endsWith("-summary") ? "visible" : "none" }}
    manageHoverState
    paint={{
      "fill-color": "cyan",
      "fill-opacity": hoverStateFilter(0.2, 0.8),
    }}
  >
    <Popup openOn="hover" let:data popupClass="popup">
      {#if data?.properties}
        <h1>{data.properties.name}</h1>
        <p style:color={colors.red}>
          Reds: {data.properties.red.toLocaleString()}
        </p>
        <p style:color={colors.amber}>
          Ambers: {data.properties.amber.toLocaleString()}
        </p>
        <p style:color={colors.green}>
          Greens: {data.properties.green.toLocaleString()}
        </p>
      {/if}
    </Popup>
  </FillLayer>
  <LineLayer
    filter={[
      "all",
      ["has", "name"],
      ["in", show == "lad-summary" ? "LAD_" : "CA_", ["get", "name"]],
    ]}
    layout={{ visibility: show.endsWith("-summary") ? "visible" : "none" }}
    paint={{
      "line-width": 5,
      "line-color": "black",
    }}
  />
</GeoJSON>

<style>
  :global(.popup .maplibregl-popup-content) {
    background-color: var(--pico-background-color);
  }
</style>
