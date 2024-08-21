#!/bin/bash

# Requires popgetter and ogr2ogr
# But doesn't worry about how to get popgetter

set -e
set -x
mkdir -p inputs
cd inputs

# Use this command to work out the the required metric ID:
# popgetter metrics -c gb_eaw -g "oa" -t "Number of cars or vans: Total: All households" | grep "Metric ID (short)"

# The metric ID for car ownership is c4149c5b at OA level

# Cannot query the metric directly because of this bug:
# https://github.com/Urban-Analytics-Technology-Platform/popgetter-cli/issues/80

# Download the data
# BBOX is the bounding box is for Newcastle upon Tyne (as a development example)
# Removing it will produce results for everywhere
popgetter data \
    --output-format geojson \
    --output-file car_ownership_epsg27000.geojson \
    --geometry-level oa \
    --id c4149c5b \
    --force-run \
    --bbox 418370,553902,434445,573095

ogr2ogr -f GPKG car_ownership.gpkg \
        -s_srs EPSG:27700 \
        -t_srs EPSG:4326 \
        car_ownership_epsg27000.geojson
