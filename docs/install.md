# Installing Scylla

There are two parts to installing Scylla:

- Installing the Scylla controller and CRDs
- Installing add-ons to fulfill Hydra traits (examples: ingress, autoscaler)

Both parts are covered in this document.

## Prerequisites

Scylla is a Kubernetes application. The Kubernetes authors consider the current version of Kubernetes and the previous version of Kubernetes to be the only two supported releases. As such, Scylla tends to only be compatible with the latest and previous Kubernetes releases. As of this writing, the supported versions of Kubernetes are 1.15 and 1.16. Your version of `kubectl` should also match your version of Kubernetes.

### Installing a Compatible Version of Kubernetes on AKS

On AKS you may install a particular version of Kubernetes. However, you need to choose a recent version of Kubernetes. Often, the default is an older release (currently, 1.13.10).

```console
$ az aks get-versions -l eastus -o table
Unable to load extension 'eventgrid'. Use --debug for more information.
KubernetesVersion    Upgrades
-------------------  ------------------------
1.15.3(preview)      None available
1.14.6               1.15.3(preview)
1.14.5               1.14.6, 1.15.3(preview)
1.13.10              1.14.5, 1.14.6
1.13.9               1.13.10, 1.14.5, 1.14.6
1.12.8               1.13.9, 1.13.10
1.12.7               1.12.8, 1.13.9, 1.13.10
1.11.10              1.12.7, 1.12.8
1.11.9               1.11.10, 1.12.7, 1.12.8
1.10.13              1.11.9, 1.11.10
1.10.12              1.10.13, 1.11.9, 1.11.10
$ az group create -l eastus -n scylla
...
$ az aks create --kubernetes-version 1.15.3 -n scylla -g scylla
...
$ az aks get-credentials -n scylla -g scylla
```

At the end of this process, verify that you are connected to this cluster with `kubectl config current-context`.

## Installing Scylla Using Helm 3

To install Helm 3, read the [Helm 3 Quickstart](https://v3.helm.sh/docs/intro/quickstart/)

> Note: In its current version, Scylla will only listen for events in one namespace. This will change in the future. For now, though, you must install Scylla into the namespace into which you will deploy Scylla apps. You may install Scylla multiple times on the same cluster as long as you deploy to a different namespace each time.
 
> Tip: As there are some breaking changes (such as Configuration => ApplicationConfiguration, Component => ComponentSchematic), if you reinstall Scylla, make sure your old CRDs are deleted. Helm will not automatically delete CRDs. You must do this with `kubectl delete crd`.

```console
$ helm install scylla ./charts/scylla --wait
NAME: scylla
LAST DEPLOYED: 2019-08-08 09:00:07.754179 -0600 MDT m=+0.710068733
NAMESPACE: default
STATUS: deployed

NOTES:
Scylla is a Kubernetes controller to manage Configuration CRDs.

It has been successfully installed.
```

This will install the CRDs and the controller into your Kubernetes cluster.

### For Older Versions of Helm

The new chart is optimized for the CRD handling introduced in Helm v3. For earlier versions of Helm you will need to manually install the CRDs:

```console
$ kubectl apply -f ./charts/scylla/crds/
```

Then you can install the Helm chart:

```console
# Helm 2:
$ helm install --name scylla ./charts/scylla
# Helm 3 alphas:
$ helm install scylla ./charts/scylla
```

### Verifying the Install

You can verify that Scylla is installed by running `kubectl get crds`:

```console
$ kubectl get crds
NAME                                      CREATED AT
applicationconfigurations.core.hydra.io   2019-10-02T19:57:32Z
componentinstances.core.hydra.io          2019-10-02T19:57:32Z
componentschematics.core.hydra.io         2019-10-02T19:57:32Z
scopes.core.hydra.io                      2019-10-02T19:57:32Z
traits.core.hydra.io                      2019-10-02T19:57:32Z
```

You should see at least those five CRDs. You can also verify that the Scylla deployment is running:

```console
$ kubectl get deployment scylla
NAME     READY   UP-TO-DATE   AVAILABLE   AGE
scylla   1/1     1            1           2m47s
```

### Uninstalling

```console
$ helm delete scylla
```

This will leave the CRDs intact.

**NOTE: When you delete the CRDs, it will delete everything touching Hydra from configurations to secrets.**

## Installing Implementations for Traits

Scylla provides several traits, including ingress and autoscaler. However, it does not install default implementations of some of these. This is because they map to primitive Kubernetes features that can be fulfilled by  different controllers.

The best place to find implementations for your traits is [Helm Hub](https://hub.helm.sh/).

### Manual Scaler

The manual scaler trait has no external dependencies.

### Ingress

To successfully use an `ingress` trait, you will need to install one of the Kubernetes Ingress controllers. We recommend [nginx-ingress](https://hub.helm.sh/charts/stable/nginx-ingress).

```console
$ helm install nginx-ingress stable/nginx-ingress
```

*Note:* You still must manage your DNS configuration as well. Mapping an ingress to `example.com` will not work if you do not also control the domain mapping for `example.com`.

### Autoscaler

To use the autoscaler trait, you must install a controller for Kubernetes `HorizontalPodAutoscaler`s. We recommend [KEDA](https://hub.helm.sh/charts/kedacore/keda-edge).

```
$ helm install keda stable/keda
```

## Running for Development

Developers may prefer to run a local copy of the Scylla daemon. To do so:

1. Make sure the CRDs are installed on your target cluster
2. Make sure your current Kubernetes context is set to your target cluster. Scylla will inherit the credentials from this context entry.
3. From the base directory of the code, run `make run`. This will start Scylla in the foreground, running locally, but listening on the remote cluster.