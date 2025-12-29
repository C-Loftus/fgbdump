clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

example:
	cargo run -- https://storage.googleapis.com/national-hydrologic-geospatial-fabric-reference-hydrofabric/reference_catchments_and_flowlines.fgb 