<script lang="ts">
  import "@picocss/pico/css/pico.jade.min.css";
  import { Layout } from "svelte-utils/two_column_layout";
  import { MapLibre, type LngLatBoundsLike } from "svelte-maplibre";
  import MapContents from "./MapContents.svelte";

  let bounds = window.location.hash
    ? undefined
    : ([-5.96, 49.89, 2.31, 55.94] as LngLatBoundsLike);

  let show: "streets" | "lad-summary" | "ca-summary" = "streets";
</script>

<Layout>
  <div slot="left">
    <h1>Pavement Parking</h1>

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
  </div>

  <div slot="main" style="position: relative; width: 100%; height: 100vh;">
    <MapLibre
      style="https://api.maptiler.com/maps/uk-openzoomstack-light/style.json?key=MZEJTanw3WpxRvt7qDfo"
      hash
      {bounds}
    >
      <MapContents {show} />
    </MapLibre>
  </div>
</Layout>
