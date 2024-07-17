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
  import { Popup } from "svelte-utils/map";
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

  let setupDone = false;
  onMount(async () => {
    await init();
    setupDone = true;
  });

  let map: Map;
  let requiredWidth = 30;
  let lanes = "scbd|ds";

  let routeGj = loadRoute();
  let routeAuthority: Feature<Polygon, { name: string; level: string }> | null =
    null;

  let resultsGj = {
    type: "FeatureCollection" as const,
    features: [],
  };
  let lanesOpacity = 80;

  function loadRoute(): FeatureCollection<LineString> {
    let x = window.localStorage.getItem("will-it-fit");
    if (x) {
      return JSON.parse(x);
    }
    return {
      type: "FeatureCollection" as const,
      features: [],
    };
  }
  $: window.localStorage.setItem("will-it-fit", JSON.stringify(routeGj));

  $: lanesGj =
    routeGj.features.length > 0 && setupDone
      ? JSON.parse(renderLanes(JSON.stringify(routeGj), lanes))
      : {
          type: "FeatureCollection" as const,
          features: [],
        };

  async function calculate() {
    try {
      resultsGj = JSON.parse(await getNegativeSpace(JSON.stringify(routeGj)));
    } catch (err) {
      window.alert(err);
    }
  }
</script>

<Layout>
  <div slot="left">
    <h1>Will it fit?</h1>

    <DrawRoute {map} bind:routeGj bind:routeAuthority />

    <hr />
    <hr />
    <hr />

    <div>
      <label>
        Required width (m):
        <input type="number" bind:value={requiredWidth} />
      </label>
    </div>
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
    <button on:click={calculate} disabled={routeGj.features.length == 0}>
      Calculate
    </button>
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
      style="https://api.maptiler.com/maps/streets/style.json?key=MZEJTanw3WpxRvt7qDfo"
      hash
      bind:map
    >
      {#if routeAuthority}
        <GeoJSON data={routeAuthority}>
          <LineLayer paint={{ "line-color": "black", "line-width": 5 }} />
        </GeoJSON>
      {/if}

      <RouteSnapperLayer />

      <GeoJSON data={routeGj}>
        <LineLayer paint={{ "line-color": "red", "line-width": 5 }} />
      </GeoJSON>
      <GeoJSON data={lanesGj}>
        <FillLayer
          paint={{
            "fill-color": ["get", "color"],
            "fill-opacity": lanesOpacity / 100,
          }}
          layout={{
            visibiliy: "none",
          }}
        />
      </GeoJSON>

      <GeoJSON data={resultsGj} generateId>
        <FillLayer
          manageHoverState
          paint={{
            "fill-color": "grey",
            "fill-opacity": hoverStateFilter(0.5, 0.8),
          }}
        >
          <Popup openOn="hover" let:props><p>{props.style}</p></Popup>
        </FillLayer>
      </GeoJSON>
    </MapLibre>
  </div>
</Layout>
