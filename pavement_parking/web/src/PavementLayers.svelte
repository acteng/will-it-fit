<script lang="ts">
  import { constructMatchExpression } from "svelte-utils/map";
  import { LineLayer, Popup, hoverStateFilter } from "svelte-maplibre";
  import { colors, type Mode, type Filters } from "./types";
  import type { ExpressionSpecification } from "maplibre-gl";

  export let show: Mode;
  export let streetFilters: Filters;
  export let sourceLayer: string | undefined;

  function makeFilter(f: Filters): ExpressionSpecification {
    let ratings = Object.entries(f.showRatings)
      .filter((pair) => pair[1])
      .map((pair) => pair[0]);
    let classes = Object.entries(f.showClasses)
      .filter((pair) => pair[1])
      .map((pair) => pair[0]);
    let directions = Object.entries(f.showDirections)
      .filter((pair) => pair[1])
      .map((pair) => pair[0]);
    return [
      "all",
      ["has", "class"],
      ["in", ["get", "class"], ["literal", classes]],
      ["in", ["get", f.useRating], ["literal", ratings]],
      ["in", ["get", "direction"], ["literal", directions]],
    ];
  }
</script>

<LineLayer
  {sourceLayer}
  filter={makeFilter(streetFilters)}
  layout={{ visibility: show == "streets" ? "visible" : "none" }}
  manageHoverState
  paint={{
    "line-width": hoverStateFilter(5, 10),
    "line-color": constructMatchExpression(
      ["get", streetFilters.useRating],
      {
        green: colors.green,
        amber: colors.amber,
        red: colors.red,
        TODO: "black",
      },
      "black",
    ),
  }}
  beforeId="Road numbers"
>
  <Popup openOn="hover" let:data popupClass="popup">
    {#if data?.properties}
      <h1>{data.properties.class} street</h1>
      <p>Direction: {data.properties.direction}</p>
      <p>
        Average road width {data.properties.average_width}, rating {data
          .properties.average_rating}
      </p>
      <p>
        Minimum road width {data.properties.minimum_width}, rating {data
          .properties.minimum_rating}
      </p>
      <p>Pavement average width: {data.properties.pavement_average_width}</p>
      <p>
        {#if data.properties.rating_change == "no_change"}
          Rating is not changed by excluding pavement parking
        {:else}
          Change: Rating including pavement parking is {data.properties
            .average_rating_inc_pavements}
        {/if}
      </p>
    {/if}
  </Popup>
</LineLayer>

<style>
  :global(.popup .maplibregl-popup-content) {
    background-color: var(--pico-background-color);
  }
</style>
