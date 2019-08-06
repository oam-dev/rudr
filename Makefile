REPO=technosophos/scylla
TAG=latest

.PHONY: build
build:
	go build -o bin/scylla ./cmd/*

.PHONY: test
test:
	go test ./cmd/... ./pkg/...

GENBIN = vendor/k8s.io/code-generator/generate-groups.sh
GENTARGET = all
GENCLIENT = github.com/microsoft/scylla/pkg/client
GENAPI = github.com/microsoft/scylla/pkg/apis
GEN_GV = core:v1alpha1
.PHONY: generate-once
generate-once:
	mkdir -p vendor/k8s.io/code-generator/hack
	# Overwrite the Kubernetes license file
	echo "/* Copyright Microsoft */" >  vendor/k8s.io/code-generator/hack/boilerplate.go.txt
	$(GENBIN) $(GENTARGET) $(GENCLIENT) $(GENAPI) $(GEN_GV)

.PHONY: docker-build
docker-build:
	docker build -t $(REPO):$(TAG) .
	docker push $(REPO) 