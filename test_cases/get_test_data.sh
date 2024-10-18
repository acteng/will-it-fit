#!/bin/bash

# This is just a convenience to download a subset of the data for test purposes.

set -e
set -x

this_dir=$(dirname $0)
AREA=Newcastle_City_Council
mkdir -p $AREA
pushd $AREA

wget https://use-land-property-data.service.gov.uk/datasets/inspire/download/$AREA.zip
unzip $AREA.zip Land_Registry_Cadastral_Parcels.gml

# Convert GML to GeoJSON, dropping all properties and fixing the coordinate system.
# See
# https://gis.stackexchange.com/questions/442709/how-can-i-debug-hanging-ogr2ogr
# for the amusing story of ignoring an unreachable schema in the GML.
ogr2ogr v1.geojson -oo DOWNLOAD_SCHEMA=NO Land_Registry_Cadastral_Parcels.gml -sql 'SELECT geometry FROM PREDEFINED'

popd

# Get the bounding boxes for all test cases
test_case_bboxes=`find $this_dir -type f -name "test_case_*_bounding_box.geojson"`

# Create a parcel file for each test case
for test_case_bbox in $test_case_bboxes; do
    echo $test_case_bbox
    output_file="${test_case_bbox/bounding_box/inspire_parcels}"
    echo $output_file
    mapshaper $AREA/v1.geojson -clip $test_case_bbox -o $output_file format=geojson geojson-type=FeatureCollection
done

# Clean up intermediate files
# mkdir -p ../inspire
# mv v2.geojson ../inspire/$AREA.geojson
# cd ..
rm -rf $AREA
