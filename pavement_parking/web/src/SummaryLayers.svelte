<script lang="ts">
  import {
    GeoJSON,
    LineLayer,
    Popup,
    hoverStateFilter,
    FillLayer,
  } from "svelte-maplibre";
  import { colors, type Mode } from "./types";

  export let show: Mode;
  export let url: string;

  let ratings = ["red", "amber", "green"] as const;
</script>

<GeoJSON data={url} generateId>
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
    beforeId="Road numbers"
  >
    <Popup openOn="hover" let:data popupClass="popup">
      {#if data?.properties}
        <h1>{data.properties.name}</h1>
        {#each ratings as rating}
          <p style:color={colors[rating]}>
            {rating}: {data.properties[`${rating}_count`].toLocaleString()} roads,
            {(data.properties[`${rating}_length`] / 1000).toFixed(2)} km
          </p>
        {/each}
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
