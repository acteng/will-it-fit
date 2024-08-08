<script lang="ts">
  import { constructMatchExpression } from "svelte-utils/map";
  import {
    LineLayer,
    GeoJSON,
    Popup,
    hoverStateFilter,
    FillLayer,
    type LngLatBoundsLike,
  } from "svelte-maplibre";

  export let show: "streets" | "lad-summary" | "ca-summary";
</script>

<GeoJSON data="/out.geojson" generateId>
  <LineLayer
    filter={["has", "class"]}
    layout={{ visibility: show == "streets" ? "visible" : "none" }}
    manageHoverState
    paint={{
      "line-width": hoverStateFilter(5, 10),
      "line-color": constructMatchExpression(
        ["get", "average_rating"],
        {
          green: "green",
          amber: "yellow",
          red: "red",
          TODO: "black",
        },
        "black",
      ),
    }}
  >
    <Popup openOn="hover" let:data popupClass="popup">
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
      <h1>{data.properties.name}</h1>
      <p>Reds: {data.properties.red.toLocaleString()}</p>
      <p>Ambers: {data.properties.amber.toLocaleString()}</p>
      <p>Greens: {data.properties.green.toLocaleString()}</p>
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
