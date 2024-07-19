#!/bin/bash
# You'll need ogr2ogr (https://gdal.org/programs/ogr2ogr.html) and mapshaper (https://github.com/mbloch/mapshaper/?tab=readme-ov-file#installation)

set -e
set -x

# TODO Run for everywhere and merge files
AREA=London_Borough_of_Southwark

wget https://use-land-property-data.service.gov.uk/datasets/inspire/download/AREA.zip
unzip $AREA.zip Land_Registry_Cadastral_Parcels.gml

# Convert GML to GeoJSON, dropping all properties and fixing the coordinate system.
# See
# https://gis.stackexchange.com/questions/442709/how-can-i-debug-hanging-ogr2ogr
# for the amusing story of ignoring an unreachable schema in the GML.
ogr2ogr v1.geojson -oo DOWNLOAD_SCHEMA=NO -t_srs EPSG:4326 Land_Registry_Cadastral_Parcels.gml -sql 'SELECT geometry FROM PREDEFINED'

# Merge adjacent polygons into one for performance. Explode the resulting multipolygon into many polygons
mapshaper v1.geojson -dissolve -explode -o v2.geojson format=geojson geojson-type=FeatureCollection

# Convert to flatgeobuf
ogr2ogr out.fgb v2.geojson

# Clean up intermediate files
rm -f $AREA.zip Land_Registry_Cadastral_Parcels.gml Land_Registry_Cadastral_Parcels.gfs v1.geojson v2.geojson

mkdir -p ../web/public
mv out.fgb ../web/public
