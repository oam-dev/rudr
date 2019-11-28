# Workloads

The *workload type* (`workloadType`) is a field in the [component schematic](./component-schematic.md) used by the developer to direct the runtime to properly execute the given component. 

OAM have two kinds of [workload types](https://github.com/oam-dev/spec/blob/master/3.component_model.md#workload-types).

* Core workload types
* Extended workload types

## Core Workloads

Core Workload types are container-based and distinguished by the following characteristics:

 - Whether they have a ***service endpoint***: an automatically assigned virtual IP address and DNS name addressable within the network scope to which the component belongs.
 - Whether they are ***replicable***: An application operator can increase or decrease the number of component replicas by applying and configuring traits when available.
 - Whether they are ***daemonized***: Rudr will attempt to restart replicas that exit, regardless of error code.

Rudr supports all of the Open Application Model [Core Workload Types](https://github.com/oam-dev/spec/blob/master/3.component_model.md#core-workload-types):

|Name|Type|Service endpoint|Replicable|Daemonized|
|-|-|-|-|-|
|[Server](#server)|core.oam.dev/v1alpha1.Server|Yes|Yes|Yes
|[Singleton Server](#singleton-server)|core.oam.dev/v1alpha1.SingletonServer|Yes|No|Yes
|[Task](#task)|core.oam.dev/v1alpha1.Task|No|Yes|No
|[Singleton Task](#singleton-task)|core.oam.dev/v1alpha1.SingletonTask|No|No|No
|[Worker](#worker)|core.oam.dev/v1alpha1.Worker|No|Yes|Yes
|[Singleton Worker](#singleton-worker)|core.oam.dev/v1alpha1.SingletonWorker|No|No|Yes

Besides Core Workloads, Rudr also support Extended Workloads. Please refer to [Extended Workloads](#extended-workloads) to learn about integrating customized workload types in Rudr.

Workload types are assigned to components as part of the [developer](https://github.com/oam-dev/spec/blob/master/2.overview_and_terminology.md#roles-and-responsibilities) role. They indicate to the [application operator](https://github.com/oam-dev/spec/blob/master/2.overview_and_terminology.md#roles-and-responsibilities) what trait(s) in the [application configuration](./application-configuration.md) that the component might require.

Workloads are named using the `GROUP/VERSION.KIND` Kubernetes convention, where `GROUP` is a uniquely named service collection, `VERSION` is an API version, and `KIND` is a group-unique name of a service. For example:

<pre>
apiVersion: core.oam.dev/v1alpha1
kind: ComponentSchematic
metadata:
  name: alpine-forever-v1
spec:
  <b style="color:blue;">workloadType:</b> core.oam.dev/v1alpha1.SingletonServer
  parameters:
    - name: message
      type: string
      required: false
    - name: unused_integer
      type: number
      required: false
      default: 5678
  containers:
    - name: runner
      image: technosophos/alpine-forever:latest
      env:
        - name: FOO
          value: bar
          fromParam: message
        - name: UNUSED
          value: "1234"
          fromParam: unused_integer
</pre>

It's important to understand that workload types don't have associated CRDs—they are simply just a field within a component. As such, Rudr users can't define custom workload types; they are limited to the workload types predefined by the platform runtime.

For more on specific workload types, refer to the sections below.

## Server

A Server is used for a long-running, scalable workload with a network endpoint and  stable name to receive network traffic for the component as a whole. Common use cases include web applications and services that expose APIs.

The Server workload in Rudr is implemented by a [Kubernetes Deployment](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/) binding with a [Kubernetes Service](https://kubernetes.io/docs/concepts/services-networking/service/).
### Workload details
| Type | Service endpoint | Replicable | Daemonized
| :-- | :--| :-- | :-- |
| `core.oam.dev/v1alpha1.Server` | &#9745; | &#9745; | &#9745; |

### Supported traits

- [Autoscaler](./traits.md#autoscaler)
- [Manual Scaler](./traits.md#manual-scaler)
- [Ingress](./traits.md#ingress)
- [Volume Mounter](./traits.md#volume-mounter)

## Singleton Server

A Singleton Server is a special case of the Server workload type that is limited to at most one replica of the container being run at a time.

> **IMPORTANT**: Operators should not attempt to modify the `replicaCount` on a `Deployment` created by a `SingletonServer`.

The Singleton Server in Rudr is implemented by a [Kubernetes Deployment](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/) binding with a [Kubernetes Service](https://kubernetes.io/docs/concepts/services-networking/service/).

### Workload details
| Type | Service endpoint | Replicable | Daemonized
| :-- | :--| :-- | :-- |
| `core.oam.dev/v1alpha1.SingletonServer` | &#9745; |  | &#9745; |

### Supported traits

- [Ingress](./traits.md#ingress)
- [Volume Mounter](./traits.md#volume-mounter)

## Task

A Task is used to run code or a script to completion. Its commonly used to run cron jobs or one-time highly parallelizable tasks that exit and free up resources upon completion.

The Task in Rudr is implemented by a [Kubernetes Job](https://kubernetes.io/docs/concepts/workloads/controllers/jobs-run-to-completion/).

### Workload details
| Type | Service endpoint | Replicable | Daemonized
| :-- | :--| :-- | :-- |
| `core.oam.dev/v1alpha1.Task` | | &#9745; | |

### Supported traits

- [Autoscaler](./traits.md#autoscaler)
- [Manual Scaler](./traits.md#manual-scaler)
- [Volume Mounter](./traits.md#volume-mounter)

## Singleton Task

A Singleton Task is a special case of the task workload type that is limited to at most one replica of the container being run at a time.

It is implemented by a [Kubernetes Job](https://kubernetes.io/docs/concepts/workloads/controllers/jobs-run-to-completion/).

### Workload details
| Type | Service endpoint | Replicable | Daemonized
| :-- | :--| :-- | :-- |
| `core.oam.dev/v1alpha1.SingletonTask` | | | |

### Supported traits

- [Volume Mounter](./traits.md#volume-mounter)

## Worker

A Worker is used for long-running, scalable workloads that do not have a service endpoint for network requests, aside from optional liveliness and readiness probe endpoints on individual replicas. Workers are typically used to pull from queues or provide other offline processing.

### Workload details
| Type | Service endpoint | Replicable | Daemonized
| :-- | :--| :-- | :-- |
| `core.oam.dev/v1alpha1.Worker` | | &#9745; | &#9745; |

### Supported traits

- [Volume Mounter](./traits.md#volume-mounter)

## Singleton Worker

A singleton worker is a special case of the worker workload type that is limited to at most one replica of the container being run at a time.

### Workload details
| Type | Service endpoint | Replicable | Daemonized
| :-- | :--| :-- | :-- |
| `core.oam.dev/v1alpha1.SingletonWorker` | | | &#9745; |

### Supported traits

- [Volume Mounter](./traits.md#volume-mounter)

## Extended Workloads

[Extended workload types](https://github.com/oam-dev/spec/blob/master/3.component_model.md#extended-workload-types) are per runtime, meaning that each runtime may define its own extended workload types. 

Extended workload types are also available in Rudr.

Rudr support two approaches for integrating extended workloads for now:

1. **Built-in Extended Workload**: this is a straightforward approach as you just need to tell Rudr about the spec of this extended workload by sending a Pull Request to Rudr code. The cons is it's less flexible as you will need to modify Rudr code for any further update and maintain it.
2. **Pluggable Extended Workload**: this is the recommended approach. The idea is you can "convert" any CRD into a Extended Workload by following certain specification. No code in Rudr is needed.

### Built-in VS Pluggable

Now Rudr has integrated OpenFaaS as built-in extended workload. For Pluggable Workload, we give [Prometheus](https://prometheus.io/) as an example.

| Built-in Extended Workloads | Pluggable Extended Workloads|
|---|---|
| [OpenFaaS](../tutorials/deploy_openfaas_workload.md)  | [Prometheus](../tutorials/deploy_prometheus_workload.md)  |   |

