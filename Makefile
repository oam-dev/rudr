REPO=technosophos/scylla
TAG=latest

.PHONY: build
build:
	cargo build

.PHONY: test
test:
	cargo test

.PHONY: docker-build
docker-build:
	docker build -t $(REPO):$(TAG) .
	docker push $(REPO) 