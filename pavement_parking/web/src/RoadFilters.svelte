<script lang="ts">
  import { colors, scenarios, type Filters } from "./types";

  export let filters: Filters;

  let classes = ["A", "B", "C", "Unclassified"] as const;
</script>

<fieldset>
  <legend>Scenario:</legend>

  {#each scenarios as [value, label]}
    <label>
      <input type="radio" {value} bind:group={filters.scenario} />
      {value}: {label}
    </label>
  {/each}
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
