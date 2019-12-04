REPO = oamdev/rudr
HEALTHREPO = oamdev/healthscope
TAG ?= latest
ARCHS := amd64 arm64
LOG_LEVEL := rudr=debug

.PHONY: build
build:
	cargo build

foo:
	echo $(PWD)

.PHONY: build-linux
build-linux: docker-build-amd64
build-linux:
	docker run -it --rm -v $(PWD)/_target:/dest $(REPO):$(TAG) cp /usr/app/rudr /dest/rudr-linux-x86_64

.PHONY: test
test:
	cargo test
	cargo clippy
	cargo test --package healthscope
	cargo clippy --package healthscope

.PHONY: run
run:
	RUST_LOG="$(LOG_LEVEL)" RUST_BACKTRACE=short cargo run

.PHONY: healthscoperun
healthscoperun:
	RUST_LOG="healthscope=debug" RUST_BACKTRACE=short cargo run --package healthscope

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

.PHONY: docker-build-arm64
docker-build-arm64:
	docker run --rm --privileged multiarch/qemu-user-static --reset -p yes
	docker build -t $(REPO)-arm64:$(TAG) --build-arg BUILDER_IMAGE=arm64v8/rust:1.38 --build-arg BASE_IMAGE=arm64v8/debian:buster-slim .
	docker build -t $(HEALTHREPO)-arm64:$(TAG) --build-arg BUILDER_IMAGE=arm64v8/rust:1.38 --build-arg BASE_IMAGE=arm64v8/debian:buster-slim --build-arg PACKAGE_NAME=healthscope .

.PHONY: docker-build-amd64
docker-build-amd64:
	docker build -t $(REPO)-amd64:$(TAG) .
	docker build -t $(HEALTHREPO)-amd64:$(TAG) --build-arg PACKAGE_NAME=healthscope .

.PHONY: docker-publish
docker-publish: docker-build-cx
	docker login -u hydraoss -p ${hydraoss_secret}
	docker push $(REPO)-amd64:$(TAG)
	docker push $(HEALTHREPO)-amd64:$(TAG)
	docker push $(REPO)-arm64:$(TAG)
	docker push $(HEALTHREPO)-arm64:$(TAG)
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest create $(REPO):$(TAG) $(REPO)-amd64:$(TAG) $(REPO)-arm64:$(TAG)
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest push $(REPO):$(TAG)
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest create $(HEALTHREPO):$(TAG) $(HEALTHREPO)-amd64:$(TAG) $(HEALTHREPO)-arm64:$(TAG)
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest push $(HEALTHREPO):$(TAG)

# Temporary while we get the ARM64 build time sorted.
.PHONY: docker-publish-amd64
docker-publish-amd64:
	docker push $(REPO)-amd64:$(TAG)
	docker push $(HEALTHREPO)-amd64:$(TAG)
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest create $(REPO):$(TAG) $(REPO)-amd64:$(TAG)
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest push $(REPO):$(TAG)
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest create $(HEALTHREPO):$(TAG) $(HEALTHREPO)-amd64:$(TAG)
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest push $(HEALTHREPO):$(TAG)

.PHONY: docker-build-dev 
docker-build-dev: 
	docker build -t $(REPO)-dev:$(TAG) .devcontainer/devenv/

.PHONY: docker-push-dev 
docker-publish-dev:
	docker login -u hydraoss -p ${hydraoss_secret}
	docker push $(REPO)-dev:$(TAG)