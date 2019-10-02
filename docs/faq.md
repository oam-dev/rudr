# Frequently Asked Questions (FAQ)

## What is the difference between Hydra and Scylla?

*Hydra* is a specification for describing applications.
*Scylla* is an implementation of the Hydra specification. Scylla runs on Kubernetes, though Hydra can be implemented on non-Kubernetes platforms.

## What do Hydra and Scylla add to the cloud native landscape?

Hydra is designed to introduce separation of concerns (SoC) into Kubernetes.

In Kubernetes today, developer information is freely intermingled with operator information. We wanted to create a way to distinguish between these two rolls so that developers could deliver an artifact that describes their microservice, and operators could apply another artifact that configures and instantiates that microservice.

In the hydra model, a `ComponentSchematic` describes a developer's view of a microservice, and an `ApplicationConfiguration` describes an application operator's view of how a component is deployed into the cluster.

## How does this compare to Knative?

We do not believe we are trying to solve the same problem as Knative.

## Can I use Hydra/Scylla to deploy existing applications?

You can describe existing applications as Hydra applications. See our [migration guide](migrating.md).

## How does this compare to Helm or Kompose?

Helm is a package manager for Kubernetes, and provides a way to package and distribute Kubernetes applications.

Hydra is just an application model that, thanks to Scylla, runs on Kubernetes. You can bundle your Hydra applications as Helm charts and deploy them using Helm.

Kompose is a tool for manipulating Kubernetes YAML documents. It is also compatible with Hydra/Scylla.

## Can I write my own traits?

Currently, all traits are built into Scylla. However, our plan is to make it possible for custom traits to be written and deployed into a Scylla cluster.

If you write a custom trait and integrate it with Scylla, consider opening a pull request. We are interested in adding more traits.

## Can I write my own scopes?

Currently, no. Scopes are fixed according to the Hydra spec.

## Does Scylla support "extended workload types" as described in the Hydra specification

No. That section of the specification is a draft, and we are not yet supporting it.