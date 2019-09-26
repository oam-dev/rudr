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

This project welcomes contributions and suggestions. See [CONTRIBUTING.md](CONTRIBUTING.md) for more details. Below are links to join the bi-weekly community meetings and our meeting notes. Community Slack channels & mailing lists will be added shortly (~ 10/1). 

| Item        | Value  |
|---------------------|---|
| Mailing List | TBD |
| Meeting Information | Bi-weekly (Starting Sept 24th), Tuesdays 10:30AM PST  |
| Meeting Link | https://zoom.us/j/623691799?pwd=ZWc4SHFNdWpRUVVNYkdJWE9zVHpjZz09   |
| Slack Channel       | TBD  |
| Meeting Notes       | https://docs.google.com/document/d/1nqdFEyULekyksFHtFvgvFAYE-0AMHKoS3RMnaKsarjs/edit?usp=sharing |

## Governance

This project follows governance structure of numerous other open source projects. See [governance.md](governance.md) for more details.

## About the Name

Scylla is one of the monsters in Homer's Odyssey. Odysseus must steer his ship between Scylla and Charybdis. Scylla is sometimes portrayed as a hydra.

## Why Rust?

On occasion, we have been asked why Scylla is written in Rust instead of Go. There is no requirement in the Kubernetes world that Kubernetes controllers be written in Go. Many languages implement the Kubernetes API and can be used for creating controllers. We decided to write Scylla in Rust because the language allows us to write Kubernetes controllers with far less code. Rust's generics make it possible to quickly and succinctly describe custom Kubernetes API resources without requiring developers to run code generators. And Rust's Kubernetes library can easily switch between Kubernetes versions with ease. We recognize that Rust might not be to everyone's taste (and neither is Go). However, we are confident that Rust is a solid choice for writing maintainable and concise Kubernetes applications.
