# Traits

A [*trait*](https://github.com/microsoft/hydra-spec/blob/master/5.traits.md) represents a piece of add-on functionality for a specific component of an application. Traits augment components with additional operational features such as traffic routing rules (including load balancing policy, network ingress routing, circuit breaking, rate limiting), auto-scaling policies, upgrade strategies, and more. As such, traits represent features of the system that are operational concerns, as opposed to developer concerns. In terms of implementation details, traits are Scylla-defined Kubernetes CRDs.

Currently, Scylla supports the following traits:

 - [Manual Scaler](#manual-scaler-trait)
 - [Autoscaler](#autoscaler-trait)
 - [Ingress](#ingress-trait)

An [App operator](https://github.com/microsoft/hydra-spec/blob/master/2.overview_and_terminology.md#roles-and-responsibilities) assigns specific traits to component workloads of an application from the [ApplicationConfiguration](application-configuration.md) manifest. For example:

<pre>
apiVersion: core.hydra.io/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: first-app
spec:
  components:
  - name: nginx-component
    instanceName: first-app-nginx
    parameterValues:
           - name: poet
           value: Eliot
           - name: poem
           value: The Wasteland
    <b style="color:blue;">traits:</b>
           - <b style="color:blue;">name:</b> ingress
             <b style="color:blue;">parameterValues:</b>
                    - name: hostname
                      value: example.com
                    - name: path
                      value: /
                    - name: service_port
                      value: 80
</pre>

You can assign a trait to a component by specifying its `name` (as listed in `kubectl get traits`) and your specific `parameterValues` (as described by `kubectl get trait <trait-name> -o yaml`). For more on using specific traits, refer to the sections below.

## Supported traits

Currently Scylla supports three traits (with more rolling out in the future, including support for defining custom traits). In order provide maximum flexibility to [Infrastructure operators](https://github.com/microsoft/hydra-spec/blob/master/2.overview_and_terminology.md#roles-and-responsibilities), however, Scylla does not install default implementations for some of these these traits. Specifically, the *Autoscaler* and *Ingress* traits require you to select and install a Kubernetes controller before you can use them in your Scylla application, since they map to primitive Kubernetes features that can be fulfilled by different controllers. You can search for implementations for your traits at [Helm Hub](https://hub.helm.sh/).

Here's how to get info on the traits supported on your Scylla installation.

**List supported traits**:

```console
`kubectl get traits`
```

**Show the schema details of a trait:**

```console
kubectl get trait <trait-name> -o yaml
````

## Manual Scaler trait

Manual Scaler trait is used to manually scale components with replicable workload types.

### Installation

None. *The manual scaler trait has no external dependencies.*

### Supported workload types

- Service
- Task


### Properties

| Name | Description | Type | Required | Default
| -- |--| -- | -- | -- |
| **replicaCount** | Number of replicas to run. | int | True | -- |

## Autoscaler trait

Autoscaler trait is used autoscale components with replicable workloads. This is implemented by the Kubernetes [Horizontal Pod Autoscaler](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/).

### Installation

To use the autoscaler trait, you must install a controller for Kubernetes `HorizontalPodAutoscaler`. We recommend using the [Kubernetes-based Event Driven Autoscaling](https://hub.helm.sh/charts/kedacore/keda-edge) (KEDA) controller:

```console
$ helm install keda stable/keda
```

### Supported workload types

- Service
- Task

### Properties

| Name | Description | Type | Required | Default |
| -- | -- | -- | -- | -- |
| **minimum** | Lower threshold of replicas to run. | int | False | `1`
| **maximum** | Higher threshold of replicas to run. Cannot be less than `minimum` value. | int | False | `10`
| **memory** | Memory consumption threshold (as percent) that will cause a scale event. | int | False | --
| **cpu** | CPU consumption threshold (as percent) that will cause a scale event. | int | False | --

## Ingress trait

Ingress trait is used for components with service workloads and provides load balancing, SSL termination and name-based virtual hosting.

### Installation

To successfully use an `ingress` trait, you will need to install one of the Kubernetes Ingress controllers. We recommend [nginx-ingress](https://hub.helm.sh/charts/stable/nginx-ingress):

```console
$ helm install nginx-ingress stable/nginx-ingress
```

*Note:* You still must manage your DNS configuration as well. Mapping an ingress to `example.com` will not work if you do not also control the domain mapping for `example.com`.

### Supported workload types

- Service
- SingletonService

### Properties

| Name | Description | Type | Required | Default |
| -- | -- | -- | -- | -- |
| **hostname** | Host name for the ingress. | string | True | --
| **service_port** | Port number on the service to bind to the ingress. | int | True | --
| **path** | Path to expose. | string | False | `/`
