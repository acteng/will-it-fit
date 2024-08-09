<script lang="ts">
  import type { Filters } from "./types";

  export let filters: Filters;

  let classes = [
    "A Road",
    "B Road",
    "Classified Unnumbered",
    "Unclassified",
    "Unknown",
    "Not Classified",
  ] as const;
</script>

<fieldset>
  <legend>Color ratings using:</legend>
  <label>
    <input type="radio" value="average_rating" bind:group={filters.useRating} />
    average width
  </label>
  <label>
    <input type="radio" value="minimum_rating" bind:group={filters.useRating} />
    minimum width
  </label>
</fieldset>

<fieldset>
  <legend>Show ratings:</legend>
  <label>
    <input type="checkbox" bind:checked={filters.showRatings.green} />
    Green
  </label>
  <label>
    <input type="checkbox" bind:checked={filters.showRatings.amber} />
    Amber
  </label>
  <label>
    <input type="checkbox" bind:checked={filters.showRatings.red} />
    Red
  </label>
</fieldset>

<fieldset>
  <legend>Show road classes:</legend>
  {#each classes as roadClass}
    <label>
      <input type="checkbox" bind:checked={filters.showClasses[roadClass]} />
      {roadClass}
    </label>
  {/each}
</fieldset>

<fieldset>
  <legend>Show road directions:</legend>
  <label>
    <input type="checkbox" bind:checked={filters.showDirections.both} />
    both ways
  </label>
  <label>
    <input type="checkbox" bind:checked={filters.showDirections["one-way"]} />
    one-way
  </label>
</fieldset>
