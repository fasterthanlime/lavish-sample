
.PHONY: all

all:
	(cd ../lavish && cargo build)
	../lavish/target/debug/lavish compile ../lavish/samples/double.lavish ./src/proto.rs
