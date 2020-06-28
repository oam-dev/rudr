# Rudr: A Kubernetes Implementation of the Open Application Model

![](https://github.com/oam-dev/rudr/workflows/Rust/badge.svg)

Rudr is an implementation of the [Open Application Model (OAM) 1.0.0-alpha1](https://github.com/oam-dev/spec/releases/tag/v1.0.0-alpha.1) for Kubernetes. 

For latest release of OAM ([1.0.0-alpha.2](https://github.com/oam-dev/spec/releases/tag/v1.0.0-alpha.2)), please check [Crossplane OAM](https://github.com/oam-dev/crossplane-oam-sample) as the reference Kubernetes implementation. Further details about OAM and Crossplane collaboration will be announced soon.

***Note: Rudr is a reference implementation for the initial working draft of the OAM specification. It does not reflect the most recent version of the OAM specification.***

## Why Rudr?

Kubernetes API resources focused on container infrastructure rather than the applications per se. Yet, application developers think in terms of application architecture, not of infrastructure.

**Rudr provides application level primitives for Kubernetes that enable:**

- The ability to define application (e.g., WordPress) in Kubernetes. 
- The ability to define operational capability (e.g., auto-scaling policy, rather than HPA) in Kubernetes.
- A portable and self-descriptive application description which includes every dependency and operational ability the application requires to run.  
- Building an application centric abstraction atop container infrastructure.

**Rudr can be used by:**

- Developers who want to describe application from developer's view, rather than learning low level primitives.
- Operators who want to focus on strategies of operating the application, rather than infrastructure details.
- Kubernetes engineers who want to define "application" in Kubernetes, or expose application level API to developers and operators, rather than full Kubernetes API.
- PaaS engineers who want to build a serverless application platform atop Kubernetes, with minimal effort.
- Software distributors who want to "define once, deploy everywhere", regardless of the differences of Kubernetes providers on multi-cloud.

## Get started

Define and deploy a [helloworld-python](./docs/how-to/create_component_from_scratch.md) application with Rudr.

## How does Rudr work?

![rudr arch](./docs/media/rudr-how-it-works.png)

Rudr defines [OAM primitives](https://github.com/oam-dev/spec/blob/master/2.overview_and_terminology.md) as Kubernetes Custom Resource Definitions (CRDs). Hence, Rudr is able to provide OAM style application level APIs including [Components](./docs/concepts/component-schematic.md) for developers to define applications, and [Traits](./docs/concepts/traits.md) for operators to define operational capabilities. Meanwhile, infra operators still work on Kubernetes itself.

Rudr controllers will maintain the mapping between OAM CRDs (e.g., Component) and Kubernetes API resources (e.g., Deployment).

## Try more things out yourself 

Read the [documentation list](./docs/README.md) for more options. Some highlights:
- [Getting started tutorials](https://github.com/oam-dev/rudr/tree/master/docs#get-started)
- [Learn Open Application Model concepts in Kubernetes](https://github.com/oam-dev/rudr/tree/master/docs#concepts)
- [Learn advanced How-To topics](https://github.com/oam-dev/rudr/tree/master/docs#how-tos)
  - For example, use Rudr with Helm and Kustmoize and migrate existing Kubernetes resources to Rudr.
- [Install and play with more workload types](https://github.com/oam-dev/rudr/tree/master/docs#extended-workloads)
- [Developing Rudr](https://github.com/oam-dev/rudr/tree/master/docs#extended-workloads)
- [FAQ](https://github.com/oam-dev/rudr/blob/master/docs/faq.md)

## More samples and demos

- [OAM samples repository](https://github.com/oam-dev/samples)

## Roadmap

Currently, Rudr relies on pre-installed workload types and traits to accomplish the task. In next release, Rudr will provide a plugin mechanism to integrate any Kubernetes Operator as workload type or operational capability. The goal is to allow users to assemble Operators ecosystem as a serverless application platform by "one click".

## Contributing

This project welcomes contributions and suggestions. See [CONTRIBUTING.md](CONTRIBUTING.md) for more details. Below are links to join the bi-weekly community meetings and our meeting notes. Community Slack channels & mailing lists will be added shortly (~ 10/1).

| Item        | Value  |
|---------------------|---|
| Mailing List | [oam-dev@@googlegroups.com](https://groups.google.com/forum/#!forum/oam-dev) |
| Meeting Information | [Bi-weekly (Starting Oct 22, 2019), Tuesdays 10:30AM PST](https://calendar.google.com/calendar?cid=dDk5YThyNGIwOWJyYTJxajNlbWI0a2FvdGtAZ3JvdXAuY2FsZW5kYXIuZ29vZ2xlLmNvbQ) |
|  | [Bi-weekly APAC (Starting Dec 24, 2019), Tuesdays 1:00PM GMT+8](https://calendar.google.com/event?action=TEMPLATE&tmeid=MzJnbHR2b3R1bHYxMG0wc2YybDJjZmhuc2pfMjAxOTEyMjRUMDUwMDAwWiBmZW5namluZ2NoYW9AbQ&tmsrc=fengjingchao%40gmail.com&scp=ALL)|
| Meeting Link | https://zoom.us/j/271516061  |
| IM Channel       | https://gitter.im/oam-dev/  |
| Meeting Notes       | [Notes](https://docs.google.com/document/d/1nqdFEyULekyksFHtFvgvFAYE-0AMHKoS3RMnaKsarjs/edit?usp=sharing) |
| Twitter      | [@oam_dev](https://twitter.com/oam_dev) |

## Governance

This project follows governance structure of numerous other open source projects. See [governance.md](governance.md) for more details.

## License

This project is available under the terms of the MIT license. See [LICENSE.txt](LICENSE.txt).
