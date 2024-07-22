#!/bin/bash
# You'll need ogr2ogr (https://gdal.org/programs/ogr2ogr.html) and mapshaper (https://github.com/mbloch/mapshaper/?tab=readme-ov-file#installation)
# This script downloads all files from INSPIRE automatically. Use with caution.

set -e

# Find all .zip files linked. 'wget -r' doesn't appear to work.
for file in `wget -q -O - https://use-land-property-data.service.gov.uk/datasets/inspire/download | grep href | grep zip | sed 's/\s\+href="//' | sed 's/"//'`; do
  wget https://use-land-property-data.service.gov.uk$file -O dataset.zip
  unzip dataset.zip Land_Registry_Cadastral_Parcels.gml

  # Convert GML to GeoJSON, dropping all properties and fixing the coordinate system.
  # See
  # https://gis.stackexchange.com/questions/442709/how-can-i-debug-hanging-ogr2ogr
  # for the amusing story of ignoring an unreachable schema in the GML.
  ogr2ogr v1.geojson -oo DOWNLOAD_SCHEMA=NO -t_srs EPSG:4326 Land_Registry_Cadastral_Parcels.gml -sql 'SELECT geometry FROM PREDEFINED'

  # Merge adjacent polygons into one for performance. Explode the resulting multipolygon into many polygons
  mapshaper v1.geojson -dissolve -explode -o v2.geojson format=geojson geojson-type=FeatureCollection

  # Clean up temporary files and rename the current one
  OUT=`basename -s .zip $file`
  mv v2.geojson $OUT.geojson
  rm -f dataset.zip Land_Registry_Cadastral_Parcels.gml v1.geojson
done
