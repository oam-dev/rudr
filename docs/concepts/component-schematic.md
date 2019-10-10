# Component Schematic

A [component schematic](https://github.com/microsoft/hydra-spec/blob/master/3.component_model.md) is a resource that declares the operational characteristics of a module of code in infrastructure neutral terms. It describes a functional code unit, or *microservice*, that can be instantiated as part of one or more larger distributed applications. You can instantiate a component by first defining its schematic (*ComponentSchematic*), [installing it](#common-operations) to your Rudr runtime, and then deploying an [application configuration](./application-configuration.md) that references it.

The *component schematic* is managed as part of the [developer](https://github.com/microsoft/hydra-spec/blob/master/2.overview_and_terminology.md#roles-and-responsibilities) role.

The key parts of a component schematic include:

- [Metadata](#metadata): Information about the component.
- [Workload type](#workload-type): Descriptor of the component's runtime profile.
- [Parameters](#parameters): (Optional). Configuration options of the component.
- [Containers](#containers): Runnable pieces of code used by the component and their resource requirements.

Here's an example application configuration (*.yaml* file):

<pre>
apiVersion: core.hydra.io/v1alpha1
kind: ComponentSchematic
<b style="color:blue;">metadata:</b>
  name: alpine-forever-v1
spec:
  <b style="color:blue;">workloadType:</b> core.hydra.io/v1alpha1.SingletonServer
  <b style="color:blue;">parameters:</b>
    - name: message
           type: string
           required: false
    - name: unused_integer
           type: number
           required: false
           default: 5678
  <b style="color:blue;">containers:</b>
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

To create a component schematic, you'll first need the [containerized code](#containers) that will constitute your microservice. From there, you might want to start with a template like the one above (or in the provided [examples](https://github.com/microsoft/scylla/tree/master/examples)) and customize to your needs.

## Common operations

Here are the key operations for installing and managing components.

**Install a component:**

```console
$ kubectl apply -f <component-schematic>.yaml
```

**List installed components:**

```console
$ kubectl get componentschematics
```

**List details of a specific component:**

```console
$ kubectl get componentschematic <component-name> -o yaml
```

**Delete a component:**

```console
$ kubectl delete componentschematic <component-name>
```

The remaining sections will walk you through the key aspects and options of a component schematic.

## Metadata

The [metadata](https://github.com/microsoft/hydra-spec/blob/master/2.overview_and_terminology.md#metadata) section provides information about the object  represented by this schematic (in this case, the `ComponentSchematic`), including its name, and optionally, any labels or annotations in the form of key/value pairs.

The metadata section consists of the following fields:

| Name | Description | Allowable values | Required | Default
| :-- | :--| :-- | :-- | :-- |
| **name** | The identifier you'll use to manage your component with kubectl (designated as `<component-name>` in the commands above). | string | &#9745; | |
| **labels** | A set of string key/value pairs assigned to the component. | Use OAM/Kubernetes [label format](https://github.com/microsoft/hydra-spec/blob/master/2.overview_and_terminology.md#label-format). | |
| **annotations** | Further metadata in key/value format describing the component, such as *version* and *description*. | Use OAM/Kubernetes [label format](https://github.com/microsoft/hydra-spec/blob/master/2.overview_and_terminology.md#label-format). `version` and `description` are pre-defined in [OAM](https://github.com/microsoft/hydra-spec/blob/master/2.overview_and_terminology.md#annotations-format) and considered best practices.  | |

Here's an example of how to specify a label and annotations:

```yaml
# Example annotations in component schematic
metadata:
  name: frontend
  labels: 
    release: canary
  annotations:
    version: v1.0.0
    description: "Frontend component of the application"
```

## Workload type

A component must declare its associated [workload type](https://github.com/microsoft/hydra-spec/blob/master/3.component_model.md#workload-types), which is an indicator to the runtime as to how the developer intends for this component to be executed. 

**See the [Workload types](../how-to/workloads.md) topic guide for more on choosing a workload type for your component.**

[Here's an example](../../examples/helloworld-python-component.yaml) declaration of the component workload type:

```yaml
workloadType: core.hydra.io/v1alpha1.Server
```

## Parameters

The (optional) [parameters](https://github.com/microsoft/hydra-spec/blob/master/3.component_model.md#parameter) section defines the configurable parameters for the component. Parameters defined here can be referenced as environment variables within  the [containerized code](#containers) of your component.

The parameters section includes the following fields:

| Name | Description | Allowable values | Required | Default
| :-- | :--| :-- | :-- | :-- |
| **name** | Identifier of the parameter | string. Must be unique per component. | &#9745; ||
| **description** | Description of the parameter. | string ||
| **type** | JSON type of the parameter. | `boolean`, `number`, `string`, or `null`| &#9745; ||
| **required**| Whether a value must be provided. | `true` or `false`||`false`|
| **default**| Default value of the parameter. | Depends on specified parameter `type`.||

[Here's an example](../../examples/env-vars.yaml) of declaring parameters and then referencing them (**`fromParam`**) as environment variables from a container:

```yaml
  # Example parameter declaration/reference in component schematic
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
```

## Containers

The [containers](https://github.com/microsoft/hydra-spec/blob/master/3.component_model.md#container) section describes the runtime configuration required to run a containerized workload for the component. 

A component schematic requires one or more containers, each consisting of the following fields:

| Name | Description | Allowable values | Required | Default
| :-- | :--| :-- | :-- | :-- |
| **name** | Name of the container. | string. Must be unique per component. | &#9745; ||
| **image**| A path or URI of the location of the container image. | string. Best practice is to include a tag suffix.| &#9745; || 
| **resources**| The runtime resources (such as CPU, memory, and storage) required by the container.| string. See [resources](#resources) section for details.||
| **ports**| The ports exposed by the container.| See [ports](#ports) section for details.||
| **cmd**| The command to run when the container starts.| string. Supply any arguments using the `args` field (see below).||
| **args**| Arguments to the `cmd` entrypoint.| string||
| **env**| Environment variables for the container.| See  [env](#env) section for details.||
| **config**| Location(s) to write configuration files within the container.| See [config](#config) section for details.||

[Here's an example](../../examples/nginx-component.yaml)  definition within the *containers* section of the component schematic:

```yaml
# Example container entry in component schematic
containers:
  - name: foo
    image: nginx:latest
    cmd:
           - nginx-debug
    args:
           - "-g"
           - "daemon off;"
    env:
           - name: TEST
             value: FOO
    config:
           - path: "/etc/access/default_user.txt"
             value: "admin"
           - path: "/etc/run/db-data"
             fromParam: "poet"
    ports:
           - type: tcp
             containerPort: 80
             name: http
```

### `resources`

The [resources](https://github.com/microsoft/hydra-spec/blob/master/3.component_model.md#resources) section describes compute resources attached to a container runtime.

The resources section includes the following fields:

| Name | Description | Allowable values | Required | Default
| :-- | :--| :-- | :-- | :-- |
| **cpu** | The minimum number of logical CPUs required for running the container. | double. (Fractional values supported.) | &#9745; | |
| **memory** | The minimum amount of memory required for running the container. | string. Use [OAM notation](https://github.com/microsoft/hydra-spec/blob/master/3.component_model.md#memory-and-disk). Must be greater than zero. | &#9745; | |
| **gpu** | The minimum number of gpus required for running this container. | double. (Fractional values supported.) | | |
| **volumes** | Specifies the attributes of the volumes that the container uses. | See [volumes](#volumes) section  for details. | |

[Here's an example](../../examples/components.yaml) resources section of the component schematic:

```yaml
# Example resources section in component schematic
resources:
  cpu:
    required: "0.5"
  memory:
    required: 100M
```

#### `volumes`

Use the [volumes](https://github.com/microsoft/hydra-spec/blob/master/3.component_model.md#volume) section to specify the volume mounts used by the container for persistent storage. 

Entries for the volumes section include the following fields:

| Name | Description | Allowable values | Required | Default
| :-- | :--| :-- | :-- | :-- |
| **name** | The name used to reference the mount.| string | &#9745; | |
| **mountPath** | Filesystem path of the mount. | string | &#9745; ||
| **sharingPolicy** | The sharing policy for the mount, indicating if it is expected to be shared or not. | `Exclusive` or `Shared`. | &#9745; ||
| **accessMode** | Access mode for the mount. | `RW` (read/write) or `RO` (read-only). | | `RW` |
| **disk** | Attributes of the underlying disk resources, including minimum `required` disk size for running the container and whether (boolean) the disk is `ephemeral`| For `required` disk size, use [OAM notation](https://github.com/microsoft/hydra-spec/blob/master/3.component_model.md#memory-and-disk). `ephemeral` takes a boolean value. | | |

[Here's an example](https://github.com/microsoft/hydra-spec/blob/master/3.component_model.md#volume) entry to the volumes section:

```yaml
# Example volume entry
volumes:
  - name: "configuration"
    mountPath: /etc/config
    accessMode: RO
    sharingPolicy: Shared
    disk:
           required: "2G" # request at least 2GB
           ephemeral: n   # non-ephemeral storage
```

### `ports`

The [ports](https://github.com/microsoft/hydra-spec/blob/master/3.component_model.md#port) section describes the ports exposed by the container.

Entries to the ports section include the following fields:

| Name | Description | Allowable values | Required | Default
| :-- | :--| :-- | :-- | :-- |
| **name** | Descriptive name for the port. | string. Must be unique per container. | &#9745; | |
| **containerPort** | The port number. | int. Must be unique per container. | &#9745; | |
| **protocol** | Transport layer protocol used by the server listening on the port. | `TCP` or `UDP` | | `TCP` |

[Here's an example](../../examples/helloworld-python-component.yaml) entry to the ports section:

```yaml
# Example port entry
ports:
  - type: tcp
    containerPort: 9999
    name: http
```

### `env`

The [env](https://github.com/microsoft/hydra-spec/blob/master/3.component_model.md#env) section describes environment variables for the container as name/value string pairs.

Entries to the env section include the following fields:

| Name | Description | Allowable values | Required | Default
| :-- | :--| :-- | :-- | :-- |
| **name** | The environment variable name. | string. Must be unique per container. | &#9745; | |
| **value** | The environment variable value. | string. If not supplied, `fromParam` must be supplied. | | |
| **fromParam** | The parameter that should be substituted into this variable as a value. | string. Name of a key/value pair defined in the [parameters](#parameters) section. | | |

[Here's an example](../../examples/env-vars.yaml) entry to the env section:

```yaml
# Example environment variable entry
env:
  - name: FOO
    value: bar
    fromParam: message # defined in parameters section of component spec
```

### `config`

The [config](https://github.com/microsoft/hydra-spec/blob/master/3.component_model.md#configfile) section describes a path to a file available within the container, as well as the data that will be written into that file. This provides a way to inject configuration files into a container.

Entries to the config section include the following fields:

| Name | Description | Allowable values | Required | Default
| :-- | :--| :-- | :-- | :-- |
| **path** | An absolute path within the container. | string | &#9745; | |
| **value** | The data to be written into the file at the specified path. | string. If this is not supplied, `fromParam` must be supplied.  |  | |
| **fromParam** | The parameter whose value should be written into this file as a value. | string. Name of a key/value pair defined in the [parameters](#parameters) section. | | |

[Here's an example](../../examples/nginx-component.yaml) entry to the config section:

```yaml
# Example config entry
config:
  - path: "/etc/access/default_user.txt"
    value: "admin"
  - path: "/etc/run/db-data"
    fromParam: "poet"
```