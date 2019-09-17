# Quickstart Guide

This guide covers how you can quickly get started using Scylla.

## Prerequisites

The following prerequisites are required for a successful use of Scylla.

1. A Kubernetes cluster
2. Deciding what security configurations to apply to your installation, if any

### Install Kubernetes or have access to a cluster

* You must have Kubernetes installed.
* You should also have a local configured copy of kubectl.

NOTE: Kubernetes versions prior to 1.6 have limited or no support for role-based access controls (RBAC).

To find out which cluster Scylla would install to, you can run `kubectl config current-context` or `kubectl cluster-info`.

```
$ kubectl config current-context
my-cluster
```

## Install Scylla

You can install Scylla using `helm`, see the [installation guide](install.md) for more details.

## Using Scylla

Once you have installed Scylla, you can start building apps. The easiest way to get going is to try out some of the [examples](../examples) in our projects.

Let's try to install the `first-app-config` as an example.

First, pre-load some component schematics:

```console
$ kubectl apply -f examples/components.yaml
```

You can now list the components available to you:

```console
$ kubectl get componentschematics
NAME                     AGE
alpine-replicable-task   19h
alpine-task              19h
hpa-example-replicated   19h
nginx-replicated         19h
nginx-singleton          19h
```

You can look at an individual component:

```console
$ kubectl get componentschematic alpine-task -o yaml
apiVersion: core.hydra.io/v1alpha1
kind: ComponentSchematic
metadata:
  creationTimestamp: "2019-08-08T03:31:36Z"
  generation: 1
  name: alpine-task
  namespace: default
  resourceVersion: "1990"
  selfLink: /apis/core.hydra.io/v1alpha1/namespaces/default/components/alpine-task
  uid: 016e40ed-8443-4a64-b87e-bdcec38e3273
spec:
  containers:
  - image: alpine:latest
    name: runner
  os: linux
  workloadType: core.hydra.io/v1alpha1.Task
```

You can also list the traits that are available on Scylla:

```console
$ kubectl get traits
NAME            AGE
autoscaler      19m
empty           19m
ingress         19m
manual-scaler   19m
```

And you can look at an individual trait in the same way that you investigate a component:

```console
$ kubectl get trait manual-scaler -o yaml
apiVersion: core.hydra.io/v1alpha1
kind: Trait
metadata:
  creationTimestamp: "2019-08-08T22:25:55Z"
  generation: 1
  name: manual-scaler
  namespace: default
  resourceVersion: "38274"
  selfLink: /apis/core.hydra.io/v1alpha1/namespaces/default/traits/manual-scaler
  uid: fecd2fb8-5f83-49dc-9a6e-dc04deaa8b92
spec:
  appliesTo:
  - replicableService
  - replicableTask
  parameters:
  - description: Number of replicas to start
    name: replicaCount
    required: true
    type: int
```

When you are ready to try installing something, take a look at the `examples/first-app-config.yaml`, which shows a basic Operational Configuration with a single trait applied:

```yaml
apiVersion: core.hydra.io/v1alpha1
kind: OperationalConfiguration
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
    traits:
      - name: ingress
        parameterValues:
          - name: hostname
            value: example.com
          - name: path
            value: /
          - name: service_port
            value: 80
```

To install this operational configuration, use `kubectl`:

Because we will use `ingress` trait here, so if your cluster don't have any kind of [ingress](https://kubernetes.io/docs/concepts/services-networking/ingress/), you should install one kind of [ingress controllers](https://kubernetes.io/docs/concepts/services-networking/ingress-controllers/) first.

Then, you need to install the component `nginx-component` as the developer do.

```console
$ kubectl apply -f examples/nginx-component.yaml
```

Finally, just install the `first-app-config` as the application operator do.

```console
$ kubectl apply -f examples/first-app-config.yaml
configuration.core.hydra.io/first-app created
```

You'll need to wait for a minute or two for it to fully deploy. Behind the scenes, Kubernetes is creating all the necessary objects.

And, of course, you can see your configuration:

```console
$ kubectl get configurations
NAME        AGE
first-app   4m23s
$ kubectl get configuration first-app -o yaml
apiVersion: core.hydra.io/v1alpha1
kind: Configuration
metadata:
  creationTimestamp: "2019-08-08T22:49:29Z"
  generation: 1
  name: first-app
  namespace: default
  resourceVersion: "40006"
  selfLink: /apis/core.hydra.io/v1alpha1/namespaces/default/configurations/first-app
  uid: 7fcb2f3f-2339-4242-8a54-6bed11d4bf86
spec:
  components:
  - instanceName: first-app-nginx
    name: nginx-singleton
    parameterValues:
    - name: poet
      value: Eliot
    - name: poem
      value: The Wasteland
    traits:
    - name: ingress
      parameterValues:
      - name: hostname
        value: example.com
      - name: path
        value: /
```

Finally, you can delete your configuration:

```console
$ kubectl delete configuration first-app
configuration.core.hydra.io "first-app" deleted
```

That will delete your application and all associated resources.

It will _not_ delete the traits and the components, which are happily awaiting your use in the next Operational Configuration.

```console
$ kubectl get traits,components
NAME                                AGE
trait.core.hydra.io/autoscaler      31m
trait.core.hydra.io/empty           31m
trait.core.hydra.io/ingress         31m
trait.core.hydra.io/manual-scaler   31m

NAME                                             AGE
component.core.hydra.io/alpine-replicable-task   19h
component.core.hydra.io/alpine-task              19h
component.core.hydra.io/hpa-example-replicated   19h
component.core.hydra.io/nginx-replicated         19h
component.core.hydra.io/nginx-singleton          19h
```

## Learn more...

Read how to [use Scylla](using_scylla.md) for more details.