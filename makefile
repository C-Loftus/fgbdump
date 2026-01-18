clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

example:
	cargo run -- testdata/colorado_subset.fgb

example_alternative_crs:
	cargo run -- testdata/colorado_subset_epsg8857.fgb

install_binary_to_path:
	cargo install --path .
