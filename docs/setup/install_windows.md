# Installing Rudr on Windows

## Prerequisites

Rudr has two dependencies, `kubectl` and `Helm 3`.

Installing necessary software packages on Windows is made easier with the package manager [Chocolatey](https://chocolatey.org/). Rudr has tw

1. To create our local development environment we will use Minikube. Install via Chocolatey by running the following commands in an Administrator shell:

        choco install minikube

        choco install kubernetes-cli

2. In order for our development environment to be compatible with Docker for Windows (and other apps that use Hyper-V) we must use a virtual network switch. Configuring the switch is straightforward: open the Hyper-V manager and from the right pane select **Network Switch Manager**. From there, create a new virtual switch with type **External**.

3. Now we can start up our VM. Be aware that Rudr is compatible with Kubernetes 1.16 and 1.17 only. From an Administrator shell run:

        minikube start --vm-driver hyperv --hyperv-virtual-switch <Name of virtual switch> --v=7 --kubernetes-version v1.17.0

    Verify the installation is working properly with: 

        kubectl get pods -n kube-system


4. To install Helm, we again can use Chocolatey:

        choco install kubernetes-helm

    Confirm the installation was successful by running:

        helm version

    You should see output displaying Helm's version info. Verify you are running version 3.x.x.

## Installing Rudr Using Helm 3

> Note: In its current version, Rudr will only listen for events in one namespace. This will change in the future. For now, though, you must install Rudr into the namespace into which you will deploy Rudr apps. You may install Rudr multiple times on the same cluster as long as you deploy to a different namespace each time.
 
> Tip: As there are some breaking changes, such as Configuration => ApplicationConfiguration, Component => ComponentSchematic, if you reinstall Rudr make sure your old CRDs are deleted. You must do this with `kubectl delete crd -l app.kubernetes.io/part-of=core.oam.dev`.

In an Administrator shell, run:

    helm install rudr ./charts/rudr


### Upgrading

To upgrade Rudr, typically you only need to use Helm.

> Tip: During the Alpha and Beta phase of Rudr, we recommend also deleting your CRDs manually. You must do this with `kubectl delete crd -l app.kubernetes.io/part-of=core.oam.devkubectl delete crd`.

```console
helm upgrade rudr charts/rudr
```

The above will update your Rudr to the latest version.

### Uninstalling

```console
helm delete rudr
```

This will leave the CRDs and configurations intact.

**NOTE: When you delete the CRDs, it will delete everything touching Open Application Model from configurations to secrets.**

```console
kubectl delete crd -l app.kubernetes.io/part-of=core.oam.dev
```

The above will delete the CRDs and clean up everything related with Open Application Model.

## Installing Implementations for Traits

Rudr provides several traits, including ingress and autoscaler. However, it does not install default implementations of some of these. This is because they map to primitive Kubernetes features that can be fulfilled by  different controllers.

The best place to find implementations for your traits is [Helm Hub](https://hub.helm.sh/).


### Manual Scaler

The manual scaler trait has no external dependencies.

### Ingress

To successfully use an `ingress` trait, you will need to install one of the Kubernetes Ingress controllers. We recommend [nginx-ingress](https://hub.helm.sh/charts/stable/nginx-ingress).

1. First, add the stable repo to your Helm installation. 

    ```Powershell
    helm repo add stable https://kubernetes-charts.storage.googleapis.com/
    ```

2. Install the NGINX ingress using Helm 3. 

    ```Powershell
    helm install nginx-ingress stable/nginx-ingress
    ```

*Note:* You still must manage your DNS configuration as well. Mapping an ingress to `example.com` will not work if you do not also control the domain mapping for `example.com`.

### Autoscaler

To use the autoscaler trait, you must install a controller for Kubernetes `HorizontalPodAutoscaler`. We recommend [KEDA](https://hub.helm.sh/charts/kedacore/keda).

1. First, add the KEDA repo to your Helm installation. 

    ```Powershell
    helm repo add kedacore https://kedacore.github.io/charts
    ```

2. Update your Helm repo. 

    ```Powershell
    helm repo update
    ```

2. Install KEDA on your cluster. 

    ```Powershell
    helm install keda kedacore/keda
    ```

## Next Steps

Deploy a sample Rudr application using the [tutorial](../tutorials/deploy_and_update.md). 

## Appendix

You could check the [appendix doc](appendix.md) to find more information.