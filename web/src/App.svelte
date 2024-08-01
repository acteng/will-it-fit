<script lang="ts">
  import "@picocss/pico/css/pico.jade.min.css";
  import { Layout } from "svelte-utils/two_column_layout";
  import {
    FillLayer,
    LineLayer,
    GeoJSON,
    MapLibre,
    hoverStateFilter,
    type Map,
  } from "svelte-maplibre";
  import { bbox, Popup } from "svelte-utils/map";
  import { Modal } from "svelte-utils";
  import { onMount } from "svelte";
  import type {
    FeatureCollection,
    LineString,
    Feature,
    Polygon,
  } from "geojson";
  import RouteSnapperLayer from "./sketch/RouteSnapperLayer.svelte";
  import init, { renderLanes, getNegativeSpace } from "backend";
  import DrawRoute from "./DrawRoute.svelte";
  import mask from "@turf/mask";

  let setupDone = false;
  onMount(async () => {
    await init();
    setupDone = true;
  });

  let map: Map;
  let showAbout = false;

  let lanes = "scbd|ds";
  let showLanes = true;
  let lanesOpacity = 80;
  let drawingRoute = false;

  let routeGj = loadRoute();
  let routeAuthority: Feature<Polygon, { name: string; level: string }> | null =
    null;

  let emptyGj = {
    type: "FeatureCollection" as const,
    features: [],
  };

  let resultsGj = emptyGj;

  function loadRoute(): FeatureCollection<LineString> {
    let x = window.localStorage.getItem("will-it-fit");
    if (x) {
      return JSON.parse(x);
    }
    return emptyGj;
  }
  $: window.localStorage.setItem("will-it-fit", JSON.stringify(routeGj));

  $: lanesGj = rerenderLanes(routeGj, setupDone, lanes);
  function rerenderLanes(
    routeGj: FeatureCollection<LineString>,
    setupDone: boolean,
    lanes: string,
  ): FeatureCollection & { width: number } {
    if (routeGj.features.length > 0 && setupDone) {
      try {
        return JSON.parse(renderLanes(JSON.stringify(routeGj), lanes));
      } catch (err) {
        window.alert(`Bad lanes config: ${err}`);
      }
    }
    return { ...emptyGj, width: 0 };
  }

  async function calculate() {
    try {
      console.time("Calculate width");
      resultsGj = JSON.parse(await getNegativeSpace(JSON.stringify(routeGj)));
      console.timeEnd("Calculate width");
    } catch (err) {
      window.alert(err);
    }
  }

  function zoomToFit() {
    map.fitBounds(bbox(routeGj), {
      padding: 20,
      animate: true,
      duration: 200,
    });
  }

  $: if (drawingRoute) {
    resultsGj = emptyGj;
    lanesGj = { ...emptyGj, width: 0 };
  }
</script>

<Layout>
  <div slot="left">
    <h1>Will it fit?</h1>
    <button on:click={() => (showAbout = true)}>About / Credits</button>

    <DrawRoute {map} bind:routeGj bind:routeAuthority bind:drawingRoute />

    <hr />
    <hr />
    <hr />

    <button on:click={zoomToFit} disabled={routeGj.features.length == 0}>
      Zoom to show route
    </button>
    <button on:click={calculate} disabled={routeGj.features.length == 0}>
      Check the width
    </button>
    <button
      on:click={() => (resultsGj = emptyGj)}
      disabled={resultsGj.features.length == 0}
    >
      Clear
    </button>

    <hr />
    <hr />
    <hr />

    <div>
      <label>
        Street features from left-to-right (
        <b>s</b>
        idewalk,
        <b>c</b>
        ycle lane,
        <b>b</b>
        us lane,
        <b>d</b>
        riving lane,
        <b>|</b>
        center line):
        <input type="text" bind:value={lanes} />
      </label>
    </div>
    <p>Required width: {lanesGj.width}m</p>
    <div>
      <label>
        <input type="checkbox" bind:checked={showLanes} />
        Show lanes
      </label>
    </div>
    <label>
      Lanes opacity: <input
        type="range"
        min="0"
        max="100"
        bind:value={lanesOpacity}
      />
    </label>
  </div>

  <div slot="main" style="position: relative; width: 100%; height: 100vh;">
    <MapLibre
      style="https://api.maptiler.com/maps/uk-openzoomstack-light/style.json?key=MZEJTanw3WpxRvt7qDfo"
      hash
      bind:map
    >
      {#if routeAuthority}
        <GeoJSON data={mask(routeAuthority)}>
          <FillLayer paint={{ "fill-color": "black", "fill-opacity": 0.5 }} />
        </GeoJSON>
      {/if}

      <RouteSnapperLayer />

      <GeoJSON data={routeGj}>
        <LineLayer
          paint={{ "line-color": "cyan", "line-width": 5 }}
          layout={{
            visibility: showLanes ? "none" : "visible",
          }}
        />
      </GeoJSON>
      <GeoJSON data={lanesGj}>
        <FillLayer
          paint={{
            "fill-color": ["get", "color"],
            "fill-opacity": lanesOpacity / 100,
          }}
          layout={{
            visibility: showLanes ? "visible" : "none",
          }}
        />
      </GeoJSON>

      <GeoJSON data={resultsGj} generateId>
        <FillLayer
          manageHoverState
          paint={{
            "fill-color": "black",
            "fill-opacity": hoverStateFilter(0.5, 0.8),
          }}
        />
        <LineLayer
          manageHoverState
          filter={["has", "width"]}
          paint={{
            "line-color": [
              "case",
              [">=", ["get", "width"], lanesGj.width],
              "green",
              "red",
            ],
            "line-width": hoverStateFilter(3, 5),
          }}
        >
          <Popup openOn="hover" let:props>
            <p>{props.width.toFixed(2)}m</p>
          </Popup>
        </LineLayer>
      </GeoJSON>
    </MapLibre>
  </div>
</Layout>

{#if showAbout}
  <Modal on:close={() => (showAbout = false)}>
    <h1>About the Will-it-fit tool</h1>

    <p>
      This is an <b>experimental</b>
      tool by
      <a href="https://github.com/dabreegster/" target="_blank">
        Dustin Carlino
      </a>
      to determine if a route with some required width will fit in between non-road
      spaces. It's an early prototype and shouldn't be used for anything yet. There
      are many caveats about the way it works and limits with the data sources that
      are not documented yet.
    </p>

    <p>
      Depending on the version of this tool you're using, you'll be seeing data
      from differnet sources. All versions use route center-lines from <a
        href="https://www.openstreetmap.org/about"
        target="_blank"
      >
        OpenStreetMap
      </a>
      .
    </p>
    <h2>
      <a
        href="https://use-land-property-data.service.gov.uk/datasets/inspire#conditions"
        target="_blank"
      >
        INSPIRE
      </a>
    </h2>
    <p>
      This information is subject to Crown copyright and database rights 2024
      and is reproduced with the permission of HM Land Registry. The polygons
      (including the associated geometry, namely x, y co-ordinates) are subject
      to Crown copyright and database rights 2024 Ordnance Survey 100026316.
    </p>

    <h2>
      <a
        href="https://www.ordnancesurvey.co.uk/products/os-mastermap-topography-layer"
        target="_blank"
      >
        Ordnance Survey MasterMap Topography Layer
      </a>
    </h2>
    <p>OS data is copyright to Ordnance Survey</p>
  </Modal>
{/if}
