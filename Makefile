
.PHONY: all run debug build doc schema

all: run doc

debug: build
	RUST_LOG=debug ./target/debug/lavish-sample

run: build
	./target/debug/lavish-sample

bench: schema
	@echo "Doing release build"
	cargo build --release
	@echo "Running sample"
	RUST_BACKTRACE=full ./target/release/lavish-sample

build: schema
	@echo "Doing debug build"
	cargo build

doc: schema
	@echo "Building documentations"
	cargo doc --no-deps

schema:
	@echo "Installing latest lavish..."
	cargo install --path ../lavish-compiler --force
	@echo "Building schema"
	lavish build src/services
