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
