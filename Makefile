
.PHONY: all

all:
	(cd ../lavish && cargo build)
	../lavish/target/debug/lavish compile src/proto