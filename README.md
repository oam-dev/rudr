# Scylla: A Kubernetes Hydra Implementation in Rust

This project implements the [Hydra specification](https://github.com/microsoft/hydra-spec) for Kubernetes.

## Building

To build:

- Make sure you have Rust 2018 Edition or newer
- Clone this repository
- Go into the main directory: `cd scylla`
- Install the CRDs in `example/crds.yaml`: `kubectl apply examples/crds.yaml`
- Run `cargo build`
- To run the server: `cargo run`

At this point, you can create, update, and destroy components using `kubectl apply`. There are two example components in `examples/`

## License

This project is available under the terms of the MIT license. See [LICENSE.txt](LICENSE.txt).

## About the Name

Scylla is one of the monsters in Homer's Odyssey. Odysseus must steer his ship between Scylla and Charybdis. Scylla is sometimes portrayed as a hydra.