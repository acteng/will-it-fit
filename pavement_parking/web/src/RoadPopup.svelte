<script lang="ts">
  import { Popup } from "svelte-maplibre";
  import { scenarios, interventions } from "./types";
</script>

<Popup openOn="hover" let:data>
  {#if data?.properties}
    <h1>{data.properties.name || "Unnamed road"}</h1>
    <p>Class: {data.properties.class} road</p>
    <p>Direction: {data.properties.direction}</p>
    <p>Length: {data.properties.length} meters</p>

    <hr />

    <p>
      Carriageway width: {data.properties.road_average_width} average, {data
        .properties.road_minimum_width} minimum
    </p>
    <p>Pavement average width: {data.properties.pavement_average_width}</p>

    <hr />

    {#each scenarios as [scenario, label]}
      <p>
        In scenario {scenario} ({label}), rating is {data.properties[
          `rating_${scenario}`
        ]}
      </p>
    {/each}
    <p>
      Intervention required: {interventions[data.properties.intervention]}
    </p>
  {/if}
</Popup>
