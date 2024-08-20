# Pavement parking analysis

1.  Run `./get_boundaries.sh` once to generate `inputs/boundaries.geojson`
2.  Modify (as needed) and run `./car_ownership.sh` to generate `inputs/car_ownership.geojson`
3.  Acquire the OS input and run `cargo run --release path/to/trn_ntwk_roadlink.gpkg`
4.  (Optionally / for large inputs) Turn that into pmtiles:

```
time tippecanoe web/public/pavements.geojson \
  --force \
  --generate-ids \
  -l pavements \
  -Z10 -z11 \
  --drop-densest-as-needed \
  --extend-zooms-if-still-dropping \
  -o web/public/pavements.pmtiles
```

5.  Set up the web app: `cd web; npm i`
6.  Run it: `npm run dev`
7.  Visit <http://localhost:5173/will-it-fit/index.html?data=/pavement.pmtiles> or <http://localhost:5173/will-it-fit/index.html?data=/out.geojson>
