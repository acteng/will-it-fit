<script lang="ts">
  import type { Map } from "maplibre-gl";
  import { onMount } from "svelte";
  import { init, RouteTool } from "route-snapper-ts";
  import { routeToolGj, snapMode, undoLength } from "./stores";

  export let map: Map;
  export let url: string;
  export let routeTool: RouteTool | null;

  let progress = 0;
  let routeToolReady = false;
  $: downloadComplete = progress >= 100;
  let failedToLoadRouteTool = false;

  onMount(async () => {
    await init();

    console.log(`Grabbing ${url}`);
    try {
      let graphBytes = await fetchWithProgress(url, (p) => {
        progress = p;
      });
      routeTool = new RouteTool(
        map,
        graphBytes,
        routeToolGj,
        snapMode,
        undoLength,
      );
      progress = 100;
      routeToolReady = true;
    } catch (err) {
      console.log(`Route tool broke: ${err}`);
      failedToLoadRouteTool = true;
    }
  });

  // Fetch a URL and return bytes. Along the way, calls setProgress with a number [0, 100] -- but sometimes over 100 when the file is compressed. This function will throw if the server doesn't send back a Content-Length header.
  async function fetchWithProgress(
    url: string,
    setProgress: (progress: number) => void,
  ): Promise<Uint8Array> {
    let response = await fetch(url);
    // TODO Handle error cases better
    let reader = response.body!.getReader();

    let lengthHeader = response.headers.get("Content-Length");
    if (!lengthHeader) {
      throw new Error(`No Content-Length header from ${url}`);
    }
    let contentLength = parseInt(lengthHeader);

    let receivedLength = 0;
    let chunks = [];
    while (true) {
      let { done, value } = await reader.read();
      if (done) {
        break;
      }

      if (value) {
        chunks.push(value);
        receivedLength += value.length;

        setProgress((100.0 * receivedLength) / contentLength);
      }
    }

    let allChunks = new Uint8Array(receivedLength);
    let position = 0;
    for (let chunk of chunks) {
      allChunks.set(chunk, position);
      position += chunk.length;
    }

    return allChunks;
  }
</script>

{#if !routeToolReady && !failedToLoadRouteTool && !downloadComplete}
  <label>
    Route tool loading
    <progress value={progress} />
  </label>
{:else if downloadComplete && !routeToolReady && !failedToLoadRouteTool}
  <label>
    Route data unpacking
    <progress />
  </label>
{:else if failedToLoadRouteTool}
  <p>Failed to load route snapper</p>
  >
{/if}

<style>
  progress {
    width: 100%;
  }
</style>
