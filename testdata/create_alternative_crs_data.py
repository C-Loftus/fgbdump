# /// script
# requires-python = ">=3.12"
# dependencies = [ "geopandas", "fiona" ]
# ///

import geopandas as gpd

# This is script for creating a flatgeobuf which can be used for
# testing on a small subset of data.
url = "https://storage.googleapis.com/national-hydrologic-geospatial-fabric-reference-hydrofabric/reference_catchments_and_flowlines.fgb"

# This is a reference dam feature with a known mainstem
feature_x = -108.2236
feature_y = 36.809000000000005

gdf = gpd.read_file(
    url,
    engine="fiona",
    bbox=[
        feature_x - 0.1,
        feature_y - 0.1,
        feature_x + 0.1,
        feature_y + 0.1,
    ],
)

# random crs for testing
gdf.to_crs("epsg:8857", inplace=True)

# this will produce a small flatgeobuf that can be used for validating
# that a feature has an associated mainstem
gdf.to_file("colorado_subset_epsg8857.fgb", driver="FlatGeobuf")