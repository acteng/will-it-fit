<script lang="ts">
  import {
    LineLayer,
    GeoJSON,
    MapLibre,
    Popup,
    hoverStateFilter,
  } from "svelte-maplibre";
  import { routeGj } from "./input";

  let resultsGj = {
    type: "FeatureCollection",
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
</script>

<h1>Will it fit?</h1>
<button on:click={snap}>Get width along route</button>

<div style="height: 90vh; position: relative">
  <MapLibre
    style="https://api.maptiler.com/maps/streets/style.json?key=MZEJTanw3WpxRvt7qDfo"
    hash
  >
    <GeoJSON data={routeGj}>
      <LineLayer paint={{ "line-color": "red", "line-width": 5 }} />
    </GeoJSON>

    <GeoJSON data={resultsGj} generateId>
      <LineLayer
        manageHoverState
        paint={{
          "line-color": "blue",
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
