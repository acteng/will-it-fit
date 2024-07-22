#!/bin/bash
# You'll need ogr2ogr (https://gdal.org/programs/ogr2ogr.html), mapshaper (https://github.com/mbloch/mapshaper/?tab=readme-ov-file#installation), and pueue (https://github.com/Nukesor/pueue)
# This script downloads all files from INSPIRE automatically. Use with caution.

set -e

# Find all .zip files linked. 'wget -r' doesn't appear to work.
for area in `wget -q -O - https://use-land-property-data.service.gov.uk/datasets/inspire/download | grep href | grep zip | sed 's/\s\+href="//' | sed 's/"//' | xargs basename -s .zip`; do
  pueue add ./inspire_one_area.sh $area
done

# Wait for pueue to finish. inspire/ will contain individual geojson files. Then:
#
# cd merge_files; cargo run --release ../inspire
#
# to produce one final fgb
