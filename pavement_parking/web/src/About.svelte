<script lang="ts">
  import { Modal } from "svelte-utils";
  import { colors } from "./types";

  let show = false;
</script>

<button on:click={() => (show = true)}>About</button>

{#if show}
  <Modal on:close={() => (show = false)}>
    <h1>About this tool</h1>

    <p>
      This uses OS MasterMap Highways Network data, using road classification
      and average and minimum width. According to the table below (thresholds
      and methodology TBD), each road is assigned a rating.
    </p>

    <table>
      <tr>
        <td></td>
        <th>A Roads</th>
        <th>B Roads</th>
        <th>C Roads</th>
        <th>Unclassified Roads</th>
      </tr>
      <tr>
        <th style:color={colors.green}>Green</th>
        <td>&ge; 11.8m</td>
        <td>&ge; 11.8m</td>
        <td>&ge; 9m</td>
        <td>&ge; 9m</td>
      </tr>
      <tr>
        <th style:color={colors.amber}>Amber</th>
        <td>10.4m - 11.8m</td>
        <td>10.4m - 11.8m</td>
        <td>7.5m - 9m</td>
        <td>7.5m - 9m</td>
      </tr>
      <tr>
        <th style:color={colors.red}>Red</th>
        <td>&lt; 10.4m</td>
        <td>&lt; 10.4m</td>
        <td>&lt; 7.5m</td>
        <td>&lt; 7.5m</td>
      </tr>
    </table>

    <p>OS data is copyright to Ordnance Survey.</p>

    <button on:click={() => (show = false)}>OK</button>
  </Modal>
{/if}

<style>
  th {
    font-weight: bold;
  }
</style>
