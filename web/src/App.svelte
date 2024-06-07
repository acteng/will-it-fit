<script lang="ts">
  import {
    LineLayer,
    GeoJSON,
    MapLibre,
    Popup,
    hoverStateFilter,
    type Map,
  } from "svelte-maplibre";
  import { routeGj } from "./input";
  import { loadAuthorities, getBestMatch } from "./match_area";
  import { onMount } from "svelte";

  onMount(loadAuthorities);

  let map: Map;
  let requiredWidth = 30;

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
      resultsGj = await resp.json();
    } catch (err) {
      window.alert(err);
    }
  }

  function getRouteSnapper() {
    window.alert(getBestMatch(map));
  }
</script>

<h1>Will it fit?</h1>
<div>
  <label>
    Required width (m):
    <input type="number" bind:value={requiredWidth} />
  </label>
</div>
<button on:click={snap}>Get width along route</button>
<button on:click={getRouteSnapper}>Get route snapper</button>

<div style="height: 90vh; position: relative">
  <MapLibre
    style="https://api.maptiler.com/maps/streets/style.json?key=MZEJTanw3WpxRvt7qDfo"
    hash
    bind:map
  >
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
