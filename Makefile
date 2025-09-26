# ---- Config -----------------------------------------------------------------
BIN         := ru-ip-dump
IMAGE       := ghcr.io/you/ru-ip-dump:latest
RUSTFLAGS  ?= -C target-cpu=native

# ---- Rust local builds -------------------------------------------------------
.PHONY: build run fmt clippy clean

build:
	@echo "==> Building release"
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --release

run: build
	@echo "==> Running (outputs: ru-ip-full.txt, ru-ip-only.txt)"
	./target/release/$(BIN)

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
	rm -f ru-ip-full.txt ru-ip-only.txt

# ---- Docker -----------------------------------------------------------------
.PHONY: docker-build docker-run docker-shell

docker-build:
	@echo "==> Building Docker image $(IMAGE)"
	docker build -t $(IMAGE) .

docker-run:
	@echo "==> Running Docker image (mount ./out -> /data)"
	mkdir -p out
	docker run --rm -v "$$(pwd)/out:/data" $(IMAGE)

docker-shell:
	docker run --rm -it -v "$$(pwd)/out:/data" --entrypoint /bin/bash $(IMAGE)
