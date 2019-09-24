# Scylla: A Kubernetes Hydra Implementation in Rust

This project implements the [Hydra specification](https://github.com/microsoft/hydra-spec) for Kubernetes.

**This is unstable, experimental, and subject to massively breaking changes. It may reflect the spec, or even features we are vetting before inclusion into the spec.**

## Install

A relatively recent version of Scylla can be installed using [Helm v3](https://github.com/helm/helm/releases).

```console
$ helm install scylla ./charts/scylla --wait
```

See the [installation guide](./docs/install.md) for more details.

## Docs

Get started with the [Quick Start](./docs/quickstart.md) guide or read the [documentation list](./docs/README.md) for more options.

## License

This project is available under the terms of the MIT license. See [LICENSE.txt](LICENSE.txt).

## Contributing

This project welcomes contributions and suggestions. See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## About the Name

Scylla is one of the monsters in Homer's Odyssey. Odysseus must steer his ship between Scylla and Charybdis. Scylla is sometimes portrayed as a hydra.
