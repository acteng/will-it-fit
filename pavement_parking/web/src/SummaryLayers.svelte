<script lang="ts">
  import {
    GeoJSON,
    LineLayer,
    Popup,
    hoverStateFilter,
    FillLayer,
  } from "svelte-maplibre";
  import {
    colors,
    scenarios,
    ratings,
    interventions,
    type Mode,
  } from "./types";

  export let show: Mode;
  export let url: string;
</script>

<GeoJSON data={url} generateId>
  <FillLayer
    filter={[
      "all",
      ["has", "name"],
      ["in", show == "lad-summary" ? "LAD_" : "CA_", ["get", "name"]],
    ]}
    layout={{ visibility: show.endsWith("-summary") ? "visible" : "none" }}
    manageHoverState
    paint={{
      "fill-color": "cyan",
      "fill-opacity": hoverStateFilter(0.2, 0.8),
    }}
    beforeId="Road numbers"
  >
    <Popup openOn="hover" let:data>
      {#if data?.properties}
        <h1>{data.properties.name}</h1>

        <table>
          <tr>
            <th>Scenario</th>
            <th style:color={colors.red}>Red</th>
            <th style:color={colors.amber}>Amber</th>
            <th style:color={colors.green}>Green</th>
          </tr>
          {#each scenarios as [scenario, label]}
            <tr>
              <th>{scenario}: {label}</th>
              {#each ratings as rating}
                {@const count = data.properties[`counts_${scenario}_${rating}`]}
                {@const length =
                  data.properties[`lengths_${scenario}_${rating}`]}
                <td>
                  {count.toLocaleString()} roads, total of {(
                    length / 1000
                  ).toFixed(2)} km
                </td>
              {/each}
            </tr>
          {/each}
        </table>

        <table>
          <tr>
            <th>Intervention</th>
            <th>Number of roads</th>
            <th>Total km</th>
          </tr>
          {#each Object.entries(interventions) as [intervention, label]}
            <tr>
              <th>{label}</th>
              <td>
                {data.properties[
                  `intervention_counts_${intervention}`
                ].toLocaleString()}
              </td>
              <td>
                {(
                  data.properties[`intervention_lengths_${intervention}`] / 1000
                ).toFixed(2)}
              </td>
            </tr>
          {/each}
        </table>
      {/if}
    </Popup>
  </FillLayer>

  <LineLayer
    filter={[
      "all",
      ["has", "name"],
      ["in", show == "lad-summary" ? "LAD_" : "CA_", ["get", "name"]],
    ]}
    layout={{ visibility: show.endsWith("-summary") ? "visible" : "none" }}
    paint={{
      "line-width": 5,
      "line-color": "black",
    }}
  />
</GeoJSON>
