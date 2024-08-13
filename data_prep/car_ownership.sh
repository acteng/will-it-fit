#!/bin/bash

# Requires popgetter
# But doesn't worry about how to get popgetter

mkdir -p car_ownership
pushd car_ownership

# Use this command to work out the the required metric ID:
# popgetter metrics -c gb_eaw -g "oa" -t "Number of cars or vans: Total: All households" | grep "Metric ID (short)"

# The metric ID for car ownership is c4149c5b at OA level

# Cannot query the metric directly because of this bug:
# https://github.com/Urban-Analytics-Technology-Platform/popgetter-cli/issues/80

# Download the data
# # BBOX is the bounding box is for Newcastle upon Tyne (as a development example)
popgetter data \
    --output-format geojson \
    --output-file car_ownership_epsg27000.geojson \
    --geometry-level oa \
    --id c4149c5b \
    --force-run \
    --bbox 422363,561108,430358,566910

mapshaper car_ownership_epsg27000.geojson -rename-fields number_of_cars_and_vans="Number of cars or vans: Total: All households" -proj init=EPSG:27700 crs=wgs84 -o car_ownership.geojson
popd
