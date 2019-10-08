# Quickstart Guide

This guide covers how to install and configure a basic Scylla installation. For more a more detailed walk-through, see the [Installation Guide](install.md)

## Prerequisites

The following prerequisites are required for a successful use of Scylla.

1. A copy of this repo (`git clone https://github.com/microsoft/scylla.git`)
2. A Kubernetes cluster version 1.15 or greater
3. `kubectl` installed and pointed at the cluster
4. [Helm 3](https://v3.helm.sh/)

To find out which cluster Scylla would install to, you can run `kubectl config current-context` or `kubectl cluster-info`.

```console
$ kubectl config current-context
my-cluster
```

## Install Scylla and Dependencies

The fastest way to install Scylla is with Helm 3.

```console
$ helm install scylla charts/scylla
NAME: scylla
LAST DEPLOYED: 2019-10-02 13:57:33.158655 -0600 MDT m=+5.183858344
NAMESPACE: default
STATUS: deployed
NOTES:
Scylla is a Kubernetes controller to manage Configuration CRDs.

It has been successfully installed.
```

This will give you a basic installation of Scylla. For the following examples, you should also install the NGINX ingress into Kubernetes:

```console
$ helm install nginx-ingress stable/nginx-ingress
NAME: nginx-ingress
LAST DEPLOYED: 2019-10-02 13:57:57.444655 -0600 MDT m=+2.129323603
NAMESPACE: default
STATUS: deployed
NOTES:
The nginx-ingress controller has been installed.
It may take a few minutes for the LoadBalancer IP to be available.
You can watch the status by running 'kubectl --namespace default get services -o wide -w nginx-ingress-controller'

...
```

This will give you a basic implementation of Kubernetes ingresses. See the [Installation Guide](install.md) for more about ingresses and other traits.

## Using Scylla

Once you have installed Scylla, you can start creating and deploying apps.

To start, install an example component:

```console
$ kubectl apply -f examples/nginx-component.yaml
```

This component declares a basic NGINX container. You can list all available components using `kubectl`:

```console
$ kubectl get componentschematics
NAME              AGE
nginx-component   14s
```

You can look at an individual component:

```console
$ kubectl get componentschematic nginx-component -o yaml
apiVersion: core.hydra.io/v1alpha1
kind: ComponentSchematic
metadata:
  creationTimestamp: "2019-10-02T20:10:36Z"
  generation: 1
  name: nginx-component-v1
  namespace: default
  resourceVersion: "119193"
  selfLink: /apis/core.hydra.io/v1alpha1/namespaces/default/componentschematics/nginx-component
  uid: a0024b12-8d56-4c4c-8fff-4092892fce76
spec:
  arch: amd64
  containers:
  - args:
    - -g
# ... more YAML
```

### Viewing Traits

Scylla provides a way to attach operational features at install time. This allows application operations an opportunity to provide functionality like autoscaling, caching, or ingress control at install time, without requiring the developer to change anything in the component.

You can also list the traits that are available on Scylla:

```console
$ kubectl get traits
NAME            AGE
autoscaler      19m
ingress         19m
manual-scaler   19m
```

And you can look at an individual trait in the same way that you investigate a component:

```console
$ kubectl get trait ingress -o yaml
apiVersion: core.hydra.io/v1alpha1
kind: Trait
metadata:
  creationTimestamp: "2019-10-02T19:57:37Z"
  generation: 1
  name: ingress
  namespace: default
  resourceVersion: "117813"
  selfLink: /apis/core.hydra.io/v1alpha1/namespaces/default/traits/ingress
  uid: 9f82c346-c8c6-4780-9949-3ecfd47879f9
spec:
  appliesTo:
  - core.hydra.io/v1alpha1.Service
  - core.hydra.io/v1alpha1.SingletonService
  properties:
  - description: Host name for the ingress
    name: hostname
    required: true
    type: string
  - description: Port number on the service
    name: service_port
    required: true
    type: int
  - description: Path to expose. Default is '/'
    name: path
    required: false
    type: string
```

The above describes a trait that attaches an ingress to a component, handling the routing of traffic to that app.

## Installing an Application Configuration

When you are ready to try installing something, take a look at the `examples/first-app-config.yaml`, which shows a basic Application Configuration with a single trait applied:

```yaml
apiVersion: core.hydra.io/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: first-app
spec:
  components:
  - name: nginx-component-v1
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

This is an example of an application composed of a singular component that has an ingress trait with an address of `example.com` and a service port of `80`. 

To install this application configuration, use `kubectl`:

```console
$ kubectl apply -f examples/first-app-config.yaml
configuration.core.hydra.io/first-app created
```

You'll need to wait for a minute or two for it to fully deploy. Behind the scenes, Scylla is creating all the necessary objects.

Once it is fully deployed, you can see your configuration:

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
    name: nginx-singleton-v1
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

## Uninstalling Applications

You can delete your configurations easily with `kubectl`:

```console
$ kubectl delete configuration first-app
configuration.core.hydra.io "first-app" deleted
```

That will delete your application and all associated resources.

It will _not_ delete the traits and the components, they are happily waiting your use in the next Application Configuration.

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