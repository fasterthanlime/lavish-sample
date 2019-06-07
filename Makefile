
.PHONY: all run doc schema

all: run doc

run: schema
	cargo build
	RUST_LOG=debug ./target/debug/lavish-sample

doc: schema
	cargo doc --no-deps

schema:
	(cd ../lavish-compiler && cargo build)
	../lavish-compiler/target/debug/lavish build src/services
