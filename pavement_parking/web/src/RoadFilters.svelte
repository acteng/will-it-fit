<script lang="ts">
  import { colors, type Filters } from "./types";

  export let filters: Filters;

  let classes = ["A", "B", "C", "Unclassified"] as const;
</script>

<fieldset>
  <legend>Color ratings using:</legend>
  <label>
    <input type="radio" value="average_rating" bind:group={filters.useRating} />
    average width (all roads)
  </label>
  <label>
    <input type="radio" value="rating_change" bind:group={filters.useRating} />
    average width (changes only)
  </label>
  <label>
    <input type="radio" value="minimum_rating" bind:group={filters.useRating} />
    minimum width (all roads)
  </label>
</fieldset>

<fieldset>
  <legend>Show ratings:</legend>
  <label style:color={colors.green}>
    <input type="checkbox" bind:checked={filters.showRatings.green} />
    Green
  </label>
  <label style:color={colors.amber}>
    <input type="checkbox" bind:checked={filters.showRatings.amber} />
    Amber
  </label>
  <label style:color={colors.red}>
    <input type="checkbox" bind:checked={filters.showRatings.red} />
    Red
  </label>
  <label>
    <!-- The color doesn't show up -->
    <input type="checkbox" bind:checked={filters.showRatings.no_change} />
    No Change
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
