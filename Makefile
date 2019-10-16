REPO = oamdev/rudr
TAG ?= latest
ARCHS := amd64 arm64
LOG_LEVEL := rudr=debug

.PHONY: build
build:
	cargo build

.PHONY: test
test:
	cargo test
	cargo clippy

.PHONY: run
run:
	RUST_LOG="$(LOG_LEVEL)" RUST_BACKTRACE=short cargo run

GIT_VERSION = $(shell git describe --always --abbrev=7 --dirty)
.PHONY: kind-e2e
kind-e2e:
	make build && \
	docker build -t $(REPO):$(GIT_VERSION) -f Dockerfile.e2e target/ && \
	kind load docker-image $(REPO):$(GIT_VERSION) \
		|| echo >&2 "kind not installed or error loading image: $(REPO):$(GIT_VERSION)" && \
	helm version && \
	helm install rudr ./charts/rudr --set image.repository=$(REPO) --set image.tag=$(GIT_VERSION) --set image.pullPolicy=IfNotPresent --wait && \
	kubectl get trait && \
	kubectl apply -f examples/components.yaml && \
	kubectl get componentschematics && \
	kubectl get componentschematic alpine-task-v1 -o yaml


.PHONY: docker-build-cx
docker-build-cx: $(addprefix docker-build-, $(ARCHS))

docker-build-arm64:
	docker run --rm --privileged multiarch/qemu-user-static --reset -p yes
	docker build -t $(REPO)-arm64:$(TAG) --build-arg BUILDER_IMAGE=arm64v8/rust:1.37 --build-arg BASE_IMAGE=arm64v8/debian:stretch-slim .

docker-build-amd64:
	docker build -t $(REPO)-amd64:$(TAG) --build-arg BUILDER_IMAGE=rust:1.37 --build-arg BASE_IMAGE=debian:stretch-slim .

.PHONY: docker-publish
docker-publish: docker-build-cx
	docker login -u hydraoss -p ${hydraoss_secret}
	docker push $(REPO)-amd64:$(TAG)
	docker push $(REPO)-arm64:$(TAG)
	export DOCKER_CLI_EXPERIMENTAL=enabled
	docker manifest create $(REPO):$(TAG) $(REPO)-amd64:$(TAG) $(REPO)-arm64:$(TAG)
	docker manifest push $(REPO):$(TAG)

# Temporary while we get the ARM64 build time sorted.
.PHONY: docker-publish-amd64
docker-publish-amd64:
	docker push $(REPO)-amd64:$(TAG)
	export DOCKER_CLI_EXPERIMENTAL=enabled
	docker manifest create $(REPO):$(TAG) $(REPO)-amd64:$(TAG)
	docker manifest push $(REPO):$(TAG)
