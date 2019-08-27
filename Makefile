REPO=hydraoss/scylla
TAG=latest

.PHONY: build
build:
	cargo build

.PHONY: test
test:
	cargo test
	cargo clippy

.PHONY: docker-build
docker-build:
	docker build -t $(REPO):$(TAG) .
	docker push $(REPO)

run:
	RUST_LOG="scylla=debug" RUST_BACKTRACE=short cargo run