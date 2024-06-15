<script lang="ts">
  import "@picocss/pico/css/pico.jade.min.css";
  import { Layout } from "svelte-utils/two_column_layout";
  import {
    LineLayer,
    FillLayer,
    GeoJSON,
    MapLibre,
    Popup,
    hoverStateFilter,
    type Map,
  } from "svelte-maplibre";
  import { loadAuthorities, getBestMatch } from "./match_area";
  import { onMount } from "svelte";
  import { init, RouteTool } from "route-snapper-ts";
  import { writable, type Writable } from "svelte/store";
  import type { Feature, Polygon } from "geojson";
  import { routeToolGj, snapMode, undoLength } from "./sketch/stores";
  import RouteSnapperLayer from "./sketch/RouteSnapperLayer.svelte";
  import RouteSnapperControls from "./sketch/RouteSnapperControls.svelte";
  import initBackend, { snapRoads, debugRoads } from "backend";
  import { renderLanes } from "./lanes";

  onMount(async () => {
    await init();
    await initBackend();
    await loadAuthorities();
  });

  let map: Map;
  let requiredWidth = 30;
  let lanes = "scbd|ds";

  let routeAuthority: Feature<Polygon> | null = null;
  let routeTool: Writable<RouteTool | null> = writable(null);
  let drawingRoute = false;

  let routeGj = {
    type: "FeatureCollection" as const,
    features: [],
  };
  let resultsGj = {
    type: "FeatureCollection" as const,
    features: [],
  };

  $: lanesGj =
    routeGj.features.length > 0
      ? renderLanes(routeGj.features[0], lanes)
      : {
          type: "FeatureCollection" as const,
          features: [],
        };

  async function snap() {
    try {
      let json = JSON.parse(await snapRoads(JSON.stringify(routeGj)));

      if (json.error) {
        resultsGj.features = [];
        window.alert(json.error);
      } else {
        resultsGj = json;
      }
    } catch (err) {
      window.alert(err);
    }
  }

  async function debug() {
    resultsGj = JSON.parse(await debugRoads(JSON.stringify(routeGj)));
  }

  async function getRouteSnapper() {
    routeAuthority = getBestMatch(map);
    let authority = `${routeAuthority.properties.level}_${routeAuthority.properties.name}`;
    let url = `https://atip.uk/route-snappers/v2.6/${authority}.bin.gz`;
    let resp = await fetch(url);
    let bytes = await resp.arrayBuffer();

    routeTool.set(
      new RouteTool(
        map,
        new Uint8Array(bytes),
        routeToolGj,
        snapMode,
        undoLength,
      ),
    );
  }

  function startDrawing(edit: boolean) {
    let copy = JSON.parse(JSON.stringify(routeGj));
    routeGj.features = [];
    resultsGj.features = [];
    drawingRoute = true;

    $routeTool!.addEventListenerSuccess((feature) => {
      routeGj.features = [feature];
      drawingRoute = false;
      $routeTool!.clearEventListeners();
    });
    $routeTool!.addEventListenerFailure(() => {
      drawingRoute = false;
      $routeTool!.clearEventListeners();
    });

    if (edit) {
      $routeTool!.editExistingRoute(copy.features[0]);
    } else {
      $routeTool!.startRoute();
    }
  }
</script>

<Layout>
  <div slot="left">
    <h1>Will it fit?</h1>

    <div>
      <button on:click={getRouteSnapper}>
        Get route snapper
        {#if routeAuthority}(currently {routeAuthority.properties.name} ({routeAuthority
            .properties.level})){/if}
      </button>
    </div>
    <div>
      <button
        on:click={() => startDrawing(false)}
        disabled={drawingRoute || $routeTool == null}
      >
        Draw a route
      </button>
      <button
        on:click={() => startDrawing(true)}
        disabled={drawingRoute ||
          $routeTool == null ||
          routeGj.features.length == 0}
      >
        Edit this route
      </button>
    </div>
    {#if drawingRoute}
      <RouteSnapperControls route_tool={$routeTool} />
    {/if}

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
    <button on:click={snap} disabled={routeGj.features.length == 0}>
      Get width along route
    </button>
    <button on:click={debug} disabled={routeGj.features.length == 0}>
      Debug roads near here
    </button>
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

      <GeoJSON data={lanesGj}>
        <FillLayer
          paint={{
            "fill-color": ["get", "color"],
          }}
        />
      </GeoJSON>

      <GeoJSON data={resultsGj} generateId>
        <LineLayer
          manageHoverState
          paint={{
            "line-color": [
              "case",
              ["<=", ["get", "avg_width"], requiredWidth],
              "red",
              "green",
            ],
            "line-width": 5,
            "line-opacity": hoverStateFilter(1.0, 0.5),
          }}
        >
          <Popup openOn="hover" let:data>
            <p>{JSON.stringify(data.properties)}</p>
          </Popup>
        </LineLayer>
      </GeoJSON>
    </MapLibre>
  </div>
</Layout>
