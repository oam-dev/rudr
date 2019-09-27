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

GIT_VERSION = $(shell git describe --always --abbrev=7 --dirty)
kind-e2e:
	make build && \
	docker build -t $(REPO):$(GIT_VERSION) -f Dockerfile.e2e target/ && \
	kind load docker-image $(REPO):$(GIT_VERSION) \
		|| echo >&2 "kind not installed or error loading image: $(REPO):$(GIT_VERSION)" && \
	helm version && \
	helm install scylla ./charts/scylla --set image.repository=$(REPO) --set image.tag=$(GIT_VERSION) --set image.pullPolicy=IfNotPresent --wait && \
	kubectl get trait && \
	kubectl apply -f examples/components.yaml && \
	kubectl get componentschematics && \
	kubectl get componentschematic alpine-task -o yaml

docker-build-arm64:
	docker run --rm --privileged multiarch/qemu-user-static --reset -p yes
	docker build -t $(REPO):arm64 -f Dockerfile.arm64 .
