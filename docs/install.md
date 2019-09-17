# Installing Scylla

## Installing Using Helm 3

> Note: In its current version, Scylla will only listen for events in one namespace. This will change in the near future.
 
> Tip: As there are some breaking changes (such as Configuration => OperationalConfiguration, Component => ComponentSchematic), if you reinstall Scylla, make sure your old CRDs are deleted.


A relatively recent version of Scylla can be installed using [Helm v3](https://github.com/helm/helm/releases).

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

*IMPORTANT:* If the installation above fails with an error `Error: apiVersion "core.hydra.io/v1alpha1" ... is not available`, wait until the CRD is cached and re-run the Helm command. The CRD cache sometimes takes a while to regenerate. 

```console
$ kubectl get trait  ## When this command returns 'No resources found' you can re-run the install
No resources found.
$ helm install scylla ./charts/scylla
```

This will install the CRDs and the controller into your Kubernetes cluster.

## For Older Versions of Helm

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

## Uninstalling

```console
$ helm delete scylla
```

This will leave the CRDs intact.

NOTE: When you delete the CRDs, it will delete everything touching Hydra from configurations to secrets.