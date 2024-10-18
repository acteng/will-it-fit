This document gives a high-level overview of the test cases which are used to explore the potential for the road space classification using Inspire polygons.

# About the Test Data

The test data includes data derived from the [Land Registry INSPIRE polygons](https://use-land-property-data.service.gov.uk/datasets/inspire). This data is used under  [Open Government Licence (OGL)](http://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/) and include these attributions:

* This information is subject to Crown copyright and database rights [2024] and is reproduced with the permission of HM Land Registry.  
* The polygons (including the associated geometry, namely x, y co-ordinates) are subject to Crown copyright and database rights [2024] Ordnance Survey AC0000851063. 

---
Additionally: Any boundaries between road space and non-road space are approximate. They have been drawn by hand, based on the visual appearance of the map purely for use in this test data, and in no way represent the actual legal boundaries.


# Grouping

## Cases A only involving whole positive INSPIRE polygons

* Case1
* Case3
* Case5

## Cases B which require sub-dividing negative space between INSPIRE polygons  

* Case 2

## Cases C involving sub-dividing positive INSPIRE polygons

* Case4


# Individual Cases

## Case 1

Great North Road, Newcastle Upon Tyne

There is a section which is dual carriageway, with a central reservation.
The northbound carriageway (west side) is mostly on unregistered land, 
but the southbound carriageway (east side) is on a series of long, thin parcels of land.

These are the parcels should be classified as road space:

```
inspire_id in (
    25855822,
    25852274,
    25853082,
    25857333,
    25853200,
    25853129,
    25853929,
)
```

Typical description of the parcels, from the Land Registry:

"""
Summary of freehold
Property description = "Land forming part of Great North Road, Gosforth, Newcastle Upon Tyne"
"""

### Success:

These parcels are correctly identified as road space. The neighbouring parcels are correctly identified at non-road space. 

## Case 2

There are residential properties which are not registered on the Land Registry, which front a road which is also on unregistered land.
The approximate location of the boundary is intuitively visible on the map.

### Success:

The polygons in `test_case_2_expected_non_road_space.geojson` are correctly identified as non-road space. 
The overlap between the identified road space and the "missing-polygons" is a small value (metric TBD).

Possible metric would be:
intersection(expected, actual) / union(expected, actual)
https://en.wikipedia.org/wiki/Jaccard_index

## Case 3

Another example of a long thin parcel of land, which is road space.

### Success:

The following parcels should be classified as road space: 
25809009

The following parcels should be classified as non-road space:
25803085
25803248
54395749

(there are other parcels within this bounding box they are not relevant for this test)


## Case 4

Pathalogical case: INSPIREID 25791501 and 25782300

This is a mixture of road space and non-road space.

* The non-road space is (in appearance) like an unregistered private property.
* The "road space" part of the polygon is the width of the road for part of the length, but then only occupies a half the width of the road for at the western end. The western end  shares the width of the road with 25782300 (which is another pathological case).
* There is an broad junction at the east end, which is close to a private property.

### Possible strategies:

Treat the whole polygon like Case #2. Then sub divide the polygon into road space and non-road space.

### Success:

The missing polygons are correctly identified as non-road space.
The overlap between the identified road space and the "missing-polygons" is a small value (metric TBD).

* The polygons in `test_case_4_expected_road_space.geojson` are correctly identified as road space. 
* The polygons in `test_case_4_expected_non_road_space.geojson` are correctly identified as non-road space. 

In both cases, we need to decide if this metric is applied to each polygon individually, or to all the polygons in aggregate.

## Case 5:

Fragment of land which has been incorporated into a junction.

INSPIREID = 25826568

This should be classified as road space.

Most other parcels in the bounding box are classified as non-road space.

### Success:

The parcel is correctly identified as road space. The neighbouring parcels are correctly identified at non-road space.
