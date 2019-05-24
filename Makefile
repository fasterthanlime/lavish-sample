
.PHONY: all

all:
	(cd ../lavish && cargo build)
	../lavish/target/debug/lavish build src/services
