<script lang="ts">
  import {
    LineLayer,
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

  onMount(async () => {
    await init();
    await loadAuthorities();
  });

  let map: Map;
  let requiredWidth = 30;

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

  async function snap() {
    try {
      let resp = await fetch("http://localhost:8080", {
        method: "POST",
        body: JSON.stringify(routeGj),
      });
      let json = await resp.json();
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

<h1>Will it fit?</h1>

<div style="border: 1px solid black; padding: 4px">
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
</div>

<div style="border: 1px solid black; padding: 4px">
  <div>
    <label>
      Required width (m):
      <input type="number" bind:value={requiredWidth} />
    </label>
  </div>
  <button on:click={snap} disabled={routeGj.features.length == 0}>
    Get width along route
  </button>
</div>

<div style="height: 90vh; position: relative">
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
      <LineLayer paint={{ "line-color": "blue", "line-width": 5 }} />
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
