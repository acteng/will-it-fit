<script lang="ts">
  import {
    GeoJSON,
    LineLayer,
    Popup,
    hoverStateFilter,
    FillLayer,
  } from "svelte-maplibre";
  import type { Mode } from "./types";

  export let show: Mode;
  export let url: string;
</script>

<GeoJSON data={url} generateId>
  <FillLayer
    layout={{ visibility: show == "census-area" ? "visible" : "none" }}
    manageHoverState
    paint={{
      "fill-color": [
        "let",
        "kerb_length_per_car_js",
        [
          "/",
          ["get", "aggregate_kerb_length"],
          ["get", "number_of_cars_and_vans"],
        ],
        [
          "interpolate-hcl",
          ["linear"],
          ["var", "kerb_length_per_car_js"],
          0,
          ["to-color", "#ff0000"],
          2.5,
          ["to-color", "#ff1111"],
          5,
          ["to-color", "#ff2222"],
          7.5,
          ["to-color", "#ff4444"],
          15,
          ["to-color", "#ffdddd"],
          20,
          ["to-color", "#ffffff"],
        ],
      ],
      "fill-opacity": hoverStateFilter(0.6, 0.9),
    }}
    beforeId="Road numbers"
  >
    <Popup openOn="hover" let:data popupClass="popup">
      {#if data?.properties}
        <h1>{data.properties.GEO_ID}</h1>
        <p>
          aggregate_kerb_length: {data.properties.aggregate_kerb_length.toLocaleString()}
          meters
        </p>
        <p>
          number_of_cars_and_vans: {data.properties.number_of_cars_and_vans.toLocaleString()}
        </p>
        <p>
          kerb_length_per_car: {data.properties.kerb_length_per_car.toLocaleString()}
          meters
        </p>
      {/if}
    </Popup>
  </FillLayer>

  <LineLayer
    layout={{ visibility: show.endsWith("-area") ? "visible" : "none" }}
    paint={{
      "line-width": 2,
      "line-color": "black",
    }}
  />
</GeoJSON>

<style>
  :global(.popup .maplibregl-popup-content) {
    background-color: var(--pico-background-color);
  }
</style>
