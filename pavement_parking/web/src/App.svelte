<script lang="ts">
  import "@picocss/pico/css/pico.jade.min.css";
  import { Layout } from "svelte-utils/two_column_layout";
  import {
    GeoJSON,
    VectorTileSource,
    MapLibre,
    type LngLatBoundsLike,
  } from "svelte-maplibre";
  import MapContents from "./MapContents.svelte";
  import StreetFilters from "./StreetFilters.svelte";
  import About from "./About.svelte";
  import { defaultFilters } from "./types";

  let bounds = window.location.hash
    ? undefined
    : ([-5.96, 49.89, 2.31, 55.94] as LngLatBoundsLike);

  let show: "streets" | "lad-summary" | "ca-summary" = "streets";
  let streetFilters = defaultFilters;

  let params = new URLSearchParams(window.location.search);
  let dataSource = params.get("data") || "";
</script>

{#if dataSource}
  <Layout>
    <div slot="left">
      <h1>Pavement Parking</h1>

      <About />

      <fieldset>
        <legend>Show:</legend>
        <label>
          <input type="radio" value="streets" bind:group={show} />
          Streets
        </label>
        <label>
          <input type="radio" value="lad-summary" bind:group={show} />
          LAD boundaries
        </label>
        <label>
          <input type="radio" value="ca-summary" bind:group={show} />
          Combined Authority boundaries
        </label>
      </fieldset>

      {#if show == "streets"}
        <StreetFilters bind:filters={streetFilters} />
      {/if}
    </div>

    <div slot="main" style="position: relative; width: 100%; height: 100vh;">
      <MapLibre
        style="https://api.maptiler.com/maps/uk-openzoomstack-light/style.json?key=MZEJTanw3WpxRvt7qDfo"
        hash
        {bounds}
      >
        {#if dataSource.endsWith(".geojson")}
          <GeoJSON data={dataSource} generateId>
            <MapContents {show} {streetFilters} sourceLayer={undefined} />
          </GeoJSON>
        {:else if dataSource.endsWith(".pmtiles")}
          <VectorTileSource url={`pmtiles://${dataSource}`}>
            <MapContents {show} {streetFilters} sourceLayer="pavement" />
          </VectorTileSource>
        {/if}
      </MapLibre>
    </div>
  </Layout>
{:else}
  <p>
    Data source not specified. If you're developing locally, try <a
      href="index.html?data=/out.geojson"
    >
      this
    </a>
    . Otherwise, you might not have access to this.
  </p>
{/if}
