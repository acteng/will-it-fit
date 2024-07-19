#!/bin/bash
# You'll need ogr2ogr (https://gdal.org/programs/ogr2ogr.html) and mapshaper (https://github.com/mbloch/mapshaper/?tab=readme-ov-file#installation)
# And access to MMTOPO (https://osdatahub.os.uk/downloads/premium/MMTOPO)

set -e
set -x

# You need to manually download OSMM topo data for some region as a gpkg and unzip.
# TODO Work with the UK-wide extract
INPUT=~/Downloads/ordnance_survey_downloads/Data/OSMasterMapTopography_6471149_topographic_area.gpkg

# Extract relevant polygons from the gpkg, drop all properties, and fix the coordinate system
ogr2ogr v1.geojson -t_srs EPSG:4326 $INPUT -sql 'SELECT geometry FROM topographic_area WHERE style_description NOT IN ("Road Or Track Fill", "Roadside Manmade Fill", "Path Fill", "Traffic Calming Fill")'

# Merge adjacent polygons into one for performance. Explode the resulting multipolygon into many polygons
mapshaper v1.geojson -dissolve -explode -o v2.geojson format=geojson geojson-type=FeatureCollection

# Convert to flatgeobuf
ogr2ogr out.fgb v2.geojson

# Clean up intermediate files
rm -f v1.geojson v2.geojson

mkdir -p ../web/public
mv out.fgb ../web/public
