# Keep private settings in .env
-include .env
export

APP = imnear

demo:
	cargo run -- "$(TEST_LAT)" "$(TEST_LON)" "$(TEST_RADIUS)" --dir "$(SAMPLE_DIR)"

demo-no-dir:
	cargo run -- "$(TEST_LAT)" "$(TEST_LON)" "$(TEST_RADIUS)"

demo-sort-by-distance:
	cargo run -- "$(TEST_LAT)" "$(TEST_LON)" "$(TEST_RADIUS)" \
		--dir "$(SAMPLE_DIR)" \
		--sort-by-distance \
		--verbose

demo-options:
	cargo run -- "$(TEST_LAT)" "$(TEST_LON)" "$(TEST_RADIUS)" \
		--dir "$(SAMPLE_DIR)" \
		--early-stop-count 5 \
		--sort-by-distance \
		--verbose

demo-pipe-in:
	cargo build
	fd --full-path .jpg "$(SAMPLE_DIR)" | "target/debug/$(APP)" "$(TEST_LAT)" "$(TEST_LON)" "$(TEST_RADIUS)" \
		--sort-by-distance \
		--verbose

demo-pipe-in-quiet:
	cargo build
	fd --full-path .jpg "$(SAMPLE_DIR)" | "target/debug/$(APP)" "$(TEST_LAT)" "$(TEST_LON)" "$(TEST_RADIUS)" \
		--sort-by-distance

demo-pipe-out-view:
	cargo build
	fd --full-path .jpg "$(SAMPLE_DIR)" \
		| "target/debug/$(APP)" "$(TEST_LAT)" "$(TEST_LON)" "$(TEST_RADIUS)" --sort-by-distance \
		| xargs open

install-link:
	rm "$(HOME)/.local/bin/$(APP)"
	ln -s "$(shell pwd)/target/debug/$(APP)" "$(HOME)/.local/bin/$(APP)"
