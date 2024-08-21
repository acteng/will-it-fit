<script lang="ts">
  import "@picocss/pico/css/pico.jade.min.css";
  import { Layout } from "svelte-utils/two_column_layout";
  import {
    GeoJSON,
    VectorTileSource,
    MapLibre,
    MapEvents,
    type LngLatBoundsLike,
    type Map,
  } from "svelte-maplibre";
  import PavementLayers from "./PavementLayers.svelte";
  import SummaryLayers from "./SummaryLayers.svelte";
  import RoadFilters from "./RoadFilters.svelte";
  import OutputAreas from "./OutputAreas.svelte";
  import About from "./About.svelte";
  import { defaultFilters, type Mode } from "./types";

  let bounds = window.location.hash
    ? undefined
    : ([-5.96, 49.89, 2.31, 55.94] as LngLatBoundsLike);

  let show: Mode = "roads";
  let roadFilters = defaultFilters;

  let params = new URLSearchParams(window.location.search);
  let pavementsUrl = params.get("data") || "";
  let summaryUrl = pavementsUrl.replace(
    pavementsUrl.endsWith(".geojson")
      ? "pavements.geojson"
      : "pavements.pmtiles",
    "summaries.geojson",
  );
  let censusUrl = pavementsUrl.replace(
    pavementsUrl.endsWith(".geojson")
      ? "pavements.geojson"
      : "pavements.pmtiles",
    "output_areas.geojson",
  );

  let map: Map;
  let zoom = 0;

  $: if (map) {
    onZoom();
  }

  function onZoom() {
    zoom = map.getZoom();
  }
</script>

{#if pavementsUrl}
  <Layout>
    <div slot="left">
      <h1>Pavement Parking</h1>

      <About />

      <fieldset>
        <legend>Show:</legend>
        <label>
          <input type="radio" value="roads" bind:group={show} />
          Roads
        </label>
        <label>
          <input type="radio" value="lad-summary" bind:group={show} />
          LAD boundaries
        </label>
        <label>
          <input type="radio" value="ca-summary" bind:group={show} />
          Combined Authority boundaries
        </label>
        <label>
          <input type="radio" value="census-area" bind:group={show} />
          Parking demand by Output Area
        </label>
      </fieldset>

      {#if show == "roads"}
        {#if zoom >= 10}
          <RoadFilters bind:filters={roadFilters} />
        {:else}
          <p>Zoom in more to see roads</p>
        {/if}
      {/if}
    </div>

    <div slot="main" style="position: relative; width: 100%; height: 100vh;">
      <MapLibre
        style="https://api.maptiler.com/maps/uk-openzoomstack-light/style.json?key=MZEJTanw3WpxRvt7qDfo"
        hash
        {bounds}
        bind:map
      >
        <MapEvents on:zoom={onZoom} />

        {#if pavementsUrl.endsWith(".geojson")}
          <GeoJSON data={pavementsUrl} generateId>
            <PavementLayers {show} {roadFilters} sourceLayer={undefined} />
          </GeoJSON>
        {:else if pavementsUrl.endsWith(".pmtiles")}
          <VectorTileSource url={`pmtiles://${pavementsUrl}`}>
            <PavementLayers {show} {roadFilters} sourceLayer="pavements" />
          </VectorTileSource>
        {/if}

        <SummaryLayers {show} url={summaryUrl} />
        <OutputAreas {show} url={censusUrl} />
      </MapLibre>
    </div>
  </Layout>
{:else}
  <p>
    Data source not specified. If you're developing locally, try <a
      href="index.html?data=/pavements.geojson"
    >
      this
    </a>
    . Otherwise, you might not have access to this.
  </p>
{/if}
