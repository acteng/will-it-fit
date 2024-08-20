#!/bin/bash

set -e
set -x

mkdir -p inputs
cd inputs

# Generated by https://github.com/acteng/atip-data-prep/blob/main/layers/boundaries.py
wget https://atip.uk/layers/v1/combined_authorities.geojson
wget https://atip.uk/layers/v1/local_authority_districts.geojson

# Clean up properties
mapshaper combined_authorities.geojson -each 'name="CA_"+name, delete CAUTH22CD' -o ca.geojson
mapshaper local_authority_districts.geojson -each 'name="LAD_"+name, delete LAD23CD' -o lads.geojson

# Merge
mapshaper -i ca.geojson lads.geojson combine-files -merge-layers -o boundaries.geojson
