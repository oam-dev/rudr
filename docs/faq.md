# Frequently Asked Questions (FAQ)

## What is the difference between Open Application Model and Rudr?

*Open Application Model* is a specification for describing applications.
*Rudr* is an implementation of the Open Application Model specification. Rudr runs on Kubernetes, though Open Application Model can be implemented on non-Kubernetes platforms.

## What do Open Application Model and Rudr add to the cloud native landscape?

Open Application Model is designed to introduce separation of concerns (SoC) into Kubernetes.

In Kubernetes today, developer information is freely intermingled with operator information. We wanted to create a way to distinguish between these two rolls so that developers could deliver an artifact that describes their microservice, and operators could apply another artifact that configures and instantiates that microservice.

In the OAM model, a `ComponentSchematic` describes a developer's view of a microservice, and an `ApplicationConfiguration` describes an application operator's view of how a component is deployed into the cluster.

## How does this compare to Knative?

We do not believe we are trying to solve the same problem as Knative.

## Can I use Open Application Model/Rudr to deploy existing applications?

You can describe existing applications as Open Application Model applications. See our [migration guide](migrating.md).

## How does this compare to Helm or Kompose?

[Helm](https://helm.sh) is a package manager for Kubernetes, and provides a way to package and distribute Kubernetes applications.

Open Application Model is just an application model that, thanks to Rudr, runs on Kubernetes. You can bundle your Open Application Model applications as Helm charts and deploy them using Helm.

[Kompose](http://kompose.io/) is a tool for manipulating Kubernetes YAML documents. It is also compatible with Open Application Model/Rudr.

## How does OAM compare with CNAB?

[Cloud Native Application Bundles (CNAB)](https://cnab.io) is a format for packaging and distributing distributed applications, including applications created using OAM. For its part, OAM does not define or prescribe a packaging format. But it works well with CNAB (as well as with Helm).

When targeting more than one implementation of OAM, developers may find CNAB a better packaging fit than Helm.

## Can I write my own traits?

Currently, all traits are built into Rudr. However, our plan is to make it possible for custom traits to be written and deployed into a Rudr cluster.

If you write a custom trait and integrate it with Rudr, consider opening a pull request. We are interested in adding more traits.

## Can I write my own scopes?

Currently, no. Scopes are fixed according to the Open Application Model spec.

## Does Rudr support "extended workload types" as described in the Open Application Model specification

No. That section of the specification is a draft, and we are not yet supporting it.


## Why Rust?

On occasion, we have been asked why Rudr is written in Rust instead of Go. There is no requirement in the Kubernetes world that Kubernetes controllers be written in Go. Many languages implement the Kubernetes API and can be used for creating controllers. We decided to write Rudr in Rust because the language allows us to write Kubernetes controllers with far less code. Rust's generics make it possible to quickly and succinctly describe custom Kubernetes API resources without requiring developers to run code generators. And Rust's Kubernetes library can easily switch between Kubernetes versions with ease. We recognize that Rust might not be to everyone's taste (and neither is Go). However, we are confident that Rust is a solid choice for writing maintainable and concise Kubernetes applications.
