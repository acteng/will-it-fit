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
        <td>&ge; ???</td>
        <td>&ge; 12.8m</td>
        <td>&ge; 10.0m</td>
        <td>&ge; 9.5m</td>
      </tr>
      <tr>
        <th style:color={colors.amber}>Amber</th>
        <td>???m - ???</td>
        <td>11.5m - 12.8m</td>
        <td>9.1m - 10.0m</td>
        <td>8.4m - 9.5m</td>
      </tr>
      <tr>
        <th style:color={colors.red}>Red</th>
        <td>&lt; ???</td>
        <td>&lt; 11.5m</td>
        <td>&lt; 9.1m</td>
        <td>&lt; 8.4m</td>
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
