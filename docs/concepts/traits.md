# Traits

A [*trait*](https://github.com/microsoft/hydra-spec/blob/master/5.traits.md) represents a piece of add-on functionality that attaches to a component instance. Traits augment components with additional operational features such as traffic routing rules (including load balancing policy, network ingress routing, circuit breaking, rate limiting), auto-scaling policies, upgrade strategies, and more. As such, traits represent features of the system that are operational concerns, as opposed to developer concerns. In terms of implementation details, traits are Rudr-defined Kubernetes CRDs.

Currently, Rudr supports the following traits:

- [Manual Scaler](#manual-scaler-trait)
- [Autoscaler](#autoscaler-trait)
- [Ingress](#ingress-trait)
- [Volume Mounter](#volume-mounter-trait)

An [application operator](https://github.com/microsoft/hydra-spec/blob/master/2.overview_and_terminology.md#roles-and-responsibilities) assigns specific traits to component workloads of an application from the [ApplicationConfiguration](application-configuration.md) manifest. For example:

<pre>
apiVersion: core.oam.dev/v1alpha1
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
        parameterValues:</b>
          - name: hostname
            value: example.com
          - name: path
            value: /
          - name: service_port
            value: 80
</pre>

You can assign a trait to a component by specifying its **`name`** (as listed in `kubectl get traits`) and your specific **Properties** (as described by `kubectl get trait <trait-name> -o yaml`). For more on using specific traits, refer to the sections below.

## Supported traits

Rudr supports several traits, with more rolling out in the future, including support for defining custom traits. In order provide maximum flexibility to [Infrastructure operators](https://github.com/microsoft/hydra-spec/blob/master/2.overview_and_terminology.md#roles-and-responsibilities), however, Rudr does not install default implementations for some of these these traits. Specifically, the *Autoscaler* and *Ingress* traits require you to select and install a Kubernetes controller before you can use them in your Rudr application, since they map to primitive Kubernetes features that can be fulfilled by different controllers. You can search for implementations for your traits at [Helm Hub](https://hub.helm.sh/).

Here's how to get info on the traits supported on your Rudr installation.

**List supported traits**:

```console
$ kubectl get traits
```

**Show the schema details of a trait:**

```console
$ kubectl get trait <trait-name> -o yaml
````

## Manual Scaler trait (`manual-scaler`)

Manual Scaler trait is used to manually scale components with replicable workload types.

### Installation

None. *The manual scaler trait has no external dependencies.*

### Supported workload types

- Server
- Task

### Properties

| Name | Description | Allowable values | Required | Default |
| :-- | :--| :-- | :-- | :-- |
| **replicaCount** | Number of replicas to run. | int | &#9745; | |

[Here's an example](../../examples/manual-scaler.yaml) of a manual scaler trait. You would attach this to a component within the application configuration:

```yaml
# Example manual scaler trait entry
traits:
  - name: manual-scaler
    parameterValues:
      - name: replicaCount
        value: 3
```

## Autoscaler trait (`autoscaler`)

Autoscaler trait is used autoscale components with replicable workloads. This is implemented by the Kubernetes [Horizontal Pod Autoscaler](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/).

### Installation

To use the autoscaler trait, you must install a controller for Kubernetes `HorizontalPodAutoscaler`. We recommend using the [Kubernetes-based Event Driven Autoscaling](https://hub.helm.sh/charts/kedacore/keda-edge) (KEDA) controller:

```console
$ helm install keda stable/keda
```

### Supported workload types

- Server
- Task

### Properties

| Name | Description | Allowable values | Required | Default |
| :-- | :--| :-- | :-- | :-- |
| **minimum** | Lower threshold of replicas to run. | int | | `1`
| **maximum** | Higher threshold of replicas to run.  | int. Cannot be less than `minimum` value. | | `10`
| **memory** | Memory consumption threshold (as percent) that will cause a scale event. | int ||
| **cpu** | CPU consumption threshold (as percent) that will cause a scale event. | int ||

[Here's an example](../../examples/autoscaler.yaml) of an autoscaler trait. You would attach this to a component within the application configuration:

```yaml
# Example autoscaler trait entry
- name: autoscaler
  parameterValues:
    - name: maximum
      value: 6
    - name: minimum
      value: 2
    - name: cpu
      value: 50
    - name: memory
      value: 50
```

## Ingress trait (`ingress`)

Ingress trait is used for components with service workloads and provides load balancing, SSL termination and name-based virtual hosting.

### Installation

To successfully use an `ingress` trait, you will need to install one of the Kubernetes Ingress controllers. We recommend [nginx-ingress](https://hub.helm.sh/charts/stable/nginx-ingress):

```console
$ helm install nginx-ingress stable/nginx-ingress
```

*Note:* You still must manage your DNS configuration as well. Mapping an ingress to `example.com` will not work if you do not also control the domain mapping for `example.com`.

### Supported workload types

- Server
- SingletonServer

### Properties

| Name | Description | Allowable values | Required | Default |
| :-- | :--| :-- | :-- | :-- |
| **hostname** | Host name for the ingress. | string | &#9745; |
| **service_port** | Port number on the service to bind to the ingress. | int. See notes below. | &#9745; | 
| **path** | Path to expose. | string | | `/`

To find your service port, you can do one of two things:

- find the port on the [ComponentSchematic](./component-schematic.md#ports)
- find the port on the desired Kubernetes [Service](https://kubernetes.io/docs/concepts/services-networking/service/) object

For example, here's how to find the port on a `ComponentSchematic`:

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ComponentSchematic
metadata:
  name: nginx-replicated-v1
spec:
  workloadType: core.oam.dev/v1alpha1.Server
  containers:
  - image: nginx:latest
    name: server
    ports:
    - containerPort: 80                  # <-- this is the service port
      name: http
      protocol: TCP
```

So to use this on an ingress, you would need to add this to your `ApplicationConfiguration`:

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: example
spec:
  components:
    - name: nginx-replicated-v1
      instanceName: example-app
      traits:
        - name: ingress
          parameterValues:
            - name: hostname
              value: example.com
            - name: path
              value: /
            - name: service_port       # <-- service_port
              value: 80                # <-- set this to the value in the component
```

Because each component may have multiple ports, the specific port must be defined in the `ApplicationConfiguration`.

## Volume Mounter trait (`volume-mounter`)

The volume mounter trait is responsible for attaching a Kubernetes [PersistentVolume Claim](https://kubernetes.io/docs/concepts/storage/persistent-volumes/#persistentvolumeclaims) (PVC) to a component.

### Installation

None. *The volume mounter trait has no external dependencies.*

### Supported workload types

- Server
- SingletonServer
- Worker
- SingletonWorker
- Task
- SingletonTask

### `paramValues`

| Name | Description | Allowable values | Required | Default |
| :-- | :--| :-- | :-- | :-- |
| **volumeName** | The name of the volume this backs. | string. Matches the volume [name](./component-schematic.md#volumes) declared in ComponentSchematic. | &#9745; |
| **storageClass** | The storage class that a PVC requires. | string. According to the available StorageClasses(s) (`kubectl get storageclass`) in your cluster and/or `default` | &#9745; |

[Here's an example](../../examples/volumes.yaml) of how to attach a storage volume to your container:

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ComponentSchematic
metadata:
  name: server-with-volume-v1
spec:
  workloadType: core.oam.dev/v1alpha1.Server
  containers:
    - name: server
      image: nginx:latest
      resources:
        volumes:
          - name: myvol
            mountPath: /myvol
            disk:
              required: "50M"
              ephemeral: true
```

In the component schematic [`volumes`](./component-schematic.md#volumes) section, one volume is specified. It must be at least `50M` in size. It is `ephemeral`, which means that the component author does not expect the data to persist if the pod is destroyed.

Sometimes, components need to persist data. In such cases, the `ephemeral` flag should be set to `false`:

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ComponentSchematic
metadata:
  name: server-with-volume-v1
spec:
  workloadType: core.oam.dev/v1alpha1.Server
  containers:
    - name: server
      image: nginx:latest
      resources:
        volumes:
          - name: myvol
            mountPath: /myvol
            disk:
              required: "50M"
              ephemeral: false
```

In the Kubernetes implementation of OAM, a Persistent Volume Claim (PVC) is used to satisfy the non-ephemeral case. However, by default Rudr does not create this PVC automatically. A trait must be applied that will indicate how the PVC is created:

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: example-server-with-volume
spec:
  components:
    - name: server-with-volume-v1
      instanceName: example-server-with-volume
      traits:
        - name: volume-mounter
          parameterValues:
            - name: volumeName
              value: myvol
            - name: storageClass
              value: default
```

The `volume-mounter` trait ensures that a PVC is created with the given name (`myvol`) using the given storage class (`default`). Typically, the `volumeName` should match the `resources.volumes[].name` field from the `ComponentSchematic`. Thus `myvol` above will match the volume declared in the `volumes` section of `server-with-volume-v1`.

When this request is processed by Rudr, it will first create the Kubernetes PVC named `myvol` and then create a Kubernetes pod that attaches that PVC as a `volumeMount`.

Attaching PVCs to Pods _may take extra time_, as the underlying system must first provision storage.
