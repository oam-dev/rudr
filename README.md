# Scylla: A Kubernetes Hydra Implementation in Rust

This project implements the [Hydra specification](https://github.com/microsoft/hydra-spec) for Kubernetes.

**This is unstable, experimental, and subject to massively breaking changes. It may reflect the spec, or even features we are vetting before inclusion into the spec.**

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

### Uninstalling

```console
$ helm delete scylla
```

This will leave the CRDs intact.

NOTE: When you delete the CRDs, it will delete everything touching Hydra from configurations to secrets.

## Using Scylla

Once you have installed Scylla, you can start building apps. The easiest way to get going is to try out some of the examples in the `examples/` directory.

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

First, you need to install the component `nginx-component` as the role developer do.

```console
$ kubectl apply -f examples/nginx-component.yaml
```

Then, just install the `first-app-config` as the role operator do.

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

## Building from Source

To build:

- Make sure you have Rust 2018 Edition or newer
- Clone this repository
- Go into the main directory: `cd scylla`
- Install the CRDs in `k8s/crds.yaml`: `kubectl apply -f k8s/crds.yaml`
- Run `cargo build`
- To run the server: `make run`, this will run scylla controller locally, and use the cluster by your `~/.kube/config`.

At this point, you will be running a local controller attached to the cluster to which your kubeconfig is pointing.

To get started, create some _components_. Components are not instantiated. They are descriptions of what things can run in your cluster.

```console
$ kubectl apply -f examples/components.yaml
component.core.hydra.io "nginx-replicated" created
component.core.hydra.io "nginx-singleton" created
$ kubectl get components
NAME               AGE
nginx-replicated   17s
nginx-singleton    17s
```

Next, create a new application that uses the component. In Hydra, as in 12-factor, an app is code (component) plus config. So you need to write a configuration. Examples are provided in the `examples/` directory:

```console
$ kubectl apply -f examples/first-app-config.yaml
```

Now you may wish to explore your cluster to see what was created:

```console
$ kubectl get configuration,pod,svc,ingress
NAME        AGE
first-app   28s

NAME                        READY     STATUS    RESTARTS   AGE
first-app-nginx-singleton   1/1       Running   0          19s

NAME                                TYPE        CLUSTER-IP    EXTERNAL-IP   PORT(S)   AGE
first-app-nginx-singleton           ClusterIP   10.0.78.193   <none>        80/TCP    19s
kubernetes                          ClusterIP   10.0.0.1      <none>        443/TCP   95d

NAME                                      HOSTS         ADDRESS   PORTS     AGE
first-app-nginx-singleton-trait-ingress   example.com             80        19s
```

To delete this, just do a `kubectl delete configuration first-app` and it will cascade and delete all of the pieces.

## License

This project is available under the terms of the MIT license. See [LICENSE.txt](LICENSE.txt).

# Contributing

This project welcomes contributions and suggestions.  Most contributions require you to agree to a Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us the rights to use your contribution. For details, visit https://cla.microsoft.com.

When you submit a pull request, a CLA-bot will automatically determine whether you need to provide a CLA and decorate the PR appropriately (e.g., label, comment). Simply follow the instructions provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/). For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

## About the Name

Scylla is one of the monsters in Homer's Odyssey. Odysseus must steer his ship between Scylla and Charybdis. Scylla is sometimes portrayed as a hydra.