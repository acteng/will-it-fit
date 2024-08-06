# Will it fit?

This is an **experimental** extension to the cross-section check tool. Given input data:

- the available width along roads
- a route
- a set of cross-sections with different minimum width requirements

Split the route and indicate what cross sections fit where.

Attempt to do all of this purely in the browser, leveraging cloud-native file formats for reading subsets of large input data.

## Developer guide

To run everything in this repo, you'll need:

- [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [ogr2ogr](https://gdal.org/programs/ogr2ogr.html)
- [mapshaper](https://github.com/mbloch/mapshaper/?tab=readme-ov-file#installation)
- [Rust](https://www.rust-lang.org/tools/install)

The web app expects `web/public/out.fgb` to be a flatgeobuffer file with polygons in WGS84 representing space that roads can't go (such as buildings). There are scripts in `data_prep/` to build these from different sources; please see there for examples.

To run the web app, `cd web; npm i` to initially install dependencies. Then `npm run wasm` whenever the Rust code in `backend/` changes. Finally, `npm run dev` starts the web app locally and automatically picks up changes in `web/`.

### Code overview

- `widths` is a library that takes a route and polygons to avoid, and generates perpendicular test lines at regular intervals along the route
- `backend` is the WASM "backend" paired with the `web` frontend
- `cli` takes an OSM PBF or XML input and calculates the width along all OSM road segments. The goal here is to compare the physical width and lane tagging, inferring street parking and other interesting questions.

All of the above needs a flatgeobuf file with polygons to treat as constraints on road width. `data_prep/` has some approaches designed to work with free [INSPIRE](https://use-land-property-data.service.gov.uk/datasets/inspire) data and non-free Ordnance Survey data.

- `data_prep/dissolver` is a WIP rewrite of the mapshaper polygon dissolve algorithm that can limit the area of output polygons
- `data_prep/fix_osmm` is a script to convert OS MasterMap mbtiles (one of the only provided formats) into flatgeobuf, for easier use elsewhere.
- `data_prep/merge_files` is a script to turn many GeoJSON files into one flatgeobuf file, used for the INSPIRE script

Finally, `pavement_parking` is checking road width and classification against a table of green/amber/red ratings to determine where pavement parking policies may need to be considered.
