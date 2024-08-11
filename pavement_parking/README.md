# Pavement parking analysis

1.  Run `./get_boundaries.sh` once to generate `inputs/boundaries.geojson`
2.  Acquire the OS input and run `cargo run --release path/to/trn_ntwk_roadlink.gpkg`
3.  (Optionally / for large inputs) Turn that into pmtiles:

```
time tippecanoe web/public/out.geojson \
  --force \
  --generate-ids \
  -l pavement \
  -Z10 -z11 \
  --drop-densest-as-needed \
  --extend-zooms-if-still-dropping \
  -o web/public/pavement.pmtiles
```

4.  Set up the web app: `cd web; npm i`
5.  Run it: `npm run dev`
6.  Visit <http://localhost:5173/will-it-fit/index.html?data=/pavement.pmtiles> or <http://localhost:5173/will-it-fit/index.html?data=/out.geojson>
