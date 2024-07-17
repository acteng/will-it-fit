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

  $: lanesGj =
    routeGj.features.length > 0 && setupDone
      ? JSON.parse(renderLanes(JSON.stringify(routeGj), lanes))
      : {
          type: "FeatureCollection" as const,
          features: [],
          width: 0,
        };

  async function calculate() {
    try {
      console.time("Calculate width");
      resultsGj = JSON.parse(await getNegativeSpace(JSON.stringify(routeGj)));
      console.timeEnd("Calculate width");
    } catch (err) {
      window.alert(err);
    }
  }

  $: if (drawingRoute) {
    resultsGj = emptyGj;
    lanesGj = emptyGj;
  }
</script>

<Layout>
  <div slot="left">
    <h1>Will it fit?</h1>

    <DrawRoute {map} bind:routeGj bind:routeAuthority bind:drawingRoute />

    <hr />
    <hr />
    <hr />

    <button on:click={calculate} disabled={routeGj.features.length == 0}>
      Check the width
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
        >
          <Popup openOn="hover" let:props><p>{props.style}</p></Popup>
        </FillLayer>
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
