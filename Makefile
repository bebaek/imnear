# Keep private settings in .env
-include .env
export

APP = imnear

test:
	cargo test -- --nocapture

build:
	cargo build

demo:
	cargo run -- --lat "$(TEST_LAT)" --lon "$(TEST_LON)" "$(TEST_RADIUS)" --dir "$(SAMPLE_DIR)"

demo-no-dir:
	cargo run -- --lat "$(TEST_LAT)" --lon "$(TEST_LON)" "$(TEST_RADIUS)"

demo-sort-by-distance:
	cargo run -- --lat "$(TEST_LAT)" --lon "$(TEST_LON)" "$(TEST_RADIUS)" \
		--dir "$(SAMPLE_DIR)" \
		--sort-by-distance \
		--verbose

demo-options:
	cargo run -- --lat "$(TEST_LAT)" --lon "$(TEST_LON)" "$(TEST_RADIUS)" \
		--dir "$(SAMPLE_DIR)" \
		--early-stop-count 5 \
		--sort-by-distance \
		--verbose

demo-pipe-in:
	cargo build
	fd --absolute-path .jpg "$(SAMPLE_DIR)" | "target/debug/$(APP)" --lat "$(TEST_LAT)" --lon "$(TEST_LON)" "$(TEST_RADIUS)" \
		--sort-by-distance \
		--verbose

demo-pipe-in-quiet:
	cargo build
	fd --absolute-path .jpg "$(SAMPLE_DIR)" | "target/debug/$(APP)" --lat "$(TEST_LAT)" --lon "$(TEST_LON)" "$(TEST_RADIUS)" \
		--sort-by-distance

demo-pipe-out-view:
	cargo build
	fd --absolute-path .jpg "$(SAMPLE_DIR)" \
		| "target/debug/$(APP)" --lat "$(TEST_LAT)" --lon "$(TEST_LON)" "$(TEST_RADIUS)" --sort-by-distance \
		| xargs -L 100 open

demo-geocode:
	cargo build
	fd --absolute-path .jpg "$(SAMPLE_DIR)" | "target/debug/$(APP)" \
		--address "$(TEST_ADDR)" \
		--sort-by-distance \
		--verbose \
		"$(TEST_RADIUS)"

install-link:
	rm "$(HOME)/.local/bin/$(APP)"
	ln -s "$(shell pwd)/target/debug/$(APP)" "$(HOME)/.local/bin/$(APP)"
