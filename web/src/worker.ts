import * as Comlink from "comlink";
import init, {
  getNegativeSpace as getNegativeSpaceInternal,
  renderLanes as renderLanesInternal,
} from "backend";
import type { Polygon, FeatureCollection, LineString } from "geojson";

export class Backend {
  setup: boolean;

  constructor() {
    this.setup = false;
  }

  async getNegativeSpace(
    routeGj: FeatureCollection<LineString>,
  ): Promise<FeatureCollection<Polygon>> {
    if (!this.setup) {
      await init();
      this.setup = true;
    }

    return JSON.parse(await getNegativeSpaceInternal(JSON.stringify(routeGj)));
  }

  async renderLanes(
    routeGj: FeatureCollection<LineString>,
    lanes: string,
  ): Promise<FeatureCollection & { width: number }> {
    if (!this.setup) {
      await init();
      this.setup = true;
    }

    return JSON.parse(renderLanesInternal(JSON.stringify(routeGj), lanes));
  }
}

Comlink.expose(Backend);
