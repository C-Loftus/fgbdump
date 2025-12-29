clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

example:
	cargo run -- testdata/colorado_subset.fgb