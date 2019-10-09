# Appendix Ⅰ: How to install a compatible version of Kubernetes on AKS?

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
 