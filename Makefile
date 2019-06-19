
.PHONY: all run debug build doc schema

all: run doc

debug: build
	RUST_LOG=debug ./target/debug/lavish-sample

run: build
	./target/debug/lavish-sample

bench: schema
	cargo build --release
	./target/release/lavish-sample

build: schema
	cargo build

doc: schema
	cargo doc --no-deps

schema:
	(cd ../lavish-compiler && cargo build)
	../lavish-compiler/target/debug/lavish build src/services
