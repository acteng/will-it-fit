#!/bin/bash
# You'll need ogr2ogr (https://gdal.org/programs/ogr2ogr.html) and mapshaper (https://github.com/mbloch/mapshaper/?tab=readme-ov-file#installation)
# This script will write ../inspire/$AREA.geojson

set -e
set -x

AREA=$1
mkdir -p $AREA
cd $AREA

wget https://use-land-property-data.service.gov.uk/datasets/inspire/download/$AREA.zip
unzip $AREA.zip Land_Registry_Cadastral_Parcels.gml

# Convert GML to GeoJSON, dropping all properties and fixing the coordinate system.
# See
# https://gis.stackexchange.com/questions/442709/how-can-i-debug-hanging-ogr2ogr
# for the amusing story of ignoring an unreachable schema in the GML.
ogr2ogr v1.geojson -oo DOWNLOAD_SCHEMA=NO -t_srs EPSG:4326 Land_Registry_Cadastral_Parcels.gml -sql 'SELECT geometry FROM PREDEFINED'

# Merge adjacent polygons into one for performance. Explode the resulting multipolygon into many polygons
mapshaper v1.geojson -dissolve -explode -o v2.geojson format=geojson geojson-type=FeatureCollection

# Clean up intermediate files
mkdir -p ../inspire
mv v2.geojson ../inspire/$AREA.geojson
cd ..
rm -rf $AREA
