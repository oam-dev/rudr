# Application Configuration

An [application configuration](https://github.com/oam-dev/spec/blob/master/6.application_configuration.md) is a resource that declares how an application is to be instantiated and configured, including parameter overrides and add-on traits. The application configuration *.yaml* file represents a logical grouping of one or more components, their operational characteristics, and runtime configuration. It is used to deploy (and update) instances of these constituent components.

The *application configuration* is managed as part of the [application operator](https://github.com/oam-dev/spec/blob/master/2.overview_and_terminology.md#roles-and-responsibilities) role.

![app config schematic comic](./images/appconfigcomic.PNG)

The key parts of an application configuration include:

- [Metadata](#metadata): Information about the installed application configuration.
- [Variables](#variables): (Optional). Variables for common parameter values across multiple components.
- [Components](#components): The constituent microservices of the application and their runtime configuration.
- [Traits](#traits): (Optional). A list of traits to enable for a given component.
- [Scopes](#scopes): (Optional). A list of scopes to apply to a given component.

Here's an example application configuration (*.yaml* file):

<pre>
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
<b style="color:blue;">metadata</b>:
  name: first-app
spec:
  <b style="color:blue;">variables</b>:
  - name: SECTION_NUMBER
    value: 3
  <b style="color:blue;">components</b>:
  - componentName: nginx-component
    instanceName: first-app-nginx
    parameterValues:
    - name: poet
      value: Eliot
    - name: poem
      value: The Wasteland
    - name: section
      value: "[fromVariable(SECTION_NUMBER)]"
    <b style="color:blue;">traits</b>:
    - name: ingress
      parameterValues:
      - name: hostname
        value: example.com
      - name: path
        value: /
      - name: service_port
        value: 80
    <b style="color: blue">applicationScopes:</b>
      - my-health-scope
</pre>

To create an application configuration, you'll first need to author the [component schematics](component-schematic.md) that will constitute your application. From there, you might want to start with a template like the one above (or in the provided [examples](https://github.com/oam-dev/rudr/tree/master/examples)) and customize to your needs.

## Common operations

Here are the key operations for installing and managing application configurations.

**Install an application configuration:**

```console
$ kubectl apply -f <application-configuration>.yaml
```

**List application configurations:**

```console
$ kubectl get configurations
```

**List details of a specific configuration:**

```console
$ kubectl get configuration <app-config-name> -o yaml
```

**Delete a configuration:**

```console
$ kubectl delete configuration <app-config-name>
```

The remaining sections will walk you through the key aspects and options of an application configuration.

## Metadata

The [metadata](https://github.com/oam-dev/spec/blob/master/2.overview_and_terminology.md#metadata) section provides information about the object  represented by this schematic (in this case, the *ApplicationConfiguration*), including its name, and optionally, any labels or annotations in the form of key/value pairs.

The metadata section consists of the following fields:

| Name | Description | Allowable values | Required | Default
| :-- | :--| :-- | :-- | :-- |
| **name** | The identifier you'll use to manage your application configuration with kubectl (designated as `<app-config-name>` in the commands above).| string | &#9745; | |
| **labels** | A set of string key/value pairs assigned to the application configuration. | Use OAM/Kubernetes [label format](https://github.com/oam-dev/spec/blob/master/2.overview_and_terminology.md#label-format). | |
| **annotations** | Further metadata in key/value format describing the application configuration, such as *version* and *description*. | Use OAM/Kubernetes [label format](https://github.com/oam-dev/spec/blob/master/2.overview_and_terminology.md#label-format). `version` and `description` are pre-defined in [OAM](https://github.com/oam-dev/spec/blob/master/2.overview_and_terminology.md#annotations-format) and considered best practices.  | |

Here's an example of how to specify a label and annotations:

```yaml
# Example label and annotations in application config
metadata:
  name: custom-app
  labels:
    release: canary
  annotations:
    version: v1.0.0
    description: "Customized version of app"
```

## Variables

The [variables](https://github.com/oam-dev/spec/blob/master/6.application_configuration.md#variable) section provides a way for an application operator to specify common values that can be substituted into multiple other locations of the application configuration.

| Name | Description | Allowable values | Required | Default
| :-- | :--| :-- | :-- | :-- |
| **name** | Name of the variable. | string. Must be unique per configuration. Includes Unicode letters, numeric characters, `_`, `-`, and `.` | &#9745; ||
| **value** | Value of the variable. | string. Any sequence of printable Unicode characters. | &#9745; ||

To declare a variable, simply provide its `name` and `value`, then reference it using **`[fromVariable(<name>)]`** syntax as needed within your component `parameterValues`:

```yaml
# Example variable declaration/reference in application config
variables:
- name: SECTION_NUMBER
  value: 3
components:
- componentName: custom-component
  instanceName: first-component
  parameterValues:
  - name: section
    value: "[fromVariable(SECTION_NUMBER)]"
```

## Components

The [components](https://github.com/oam-dev/spec/blob/master/6.application_configuration.md#component) section defines the instances of components to create with the application. 

An application configuration requires one or more listed components, each consisting of the following fields:

| Name | Description | Allowable values | Required | Default
| :-- | :--| :-- | :-- | :-- |
| **componentName** | Name of the [ComponentSchematic](./component-schematic.md) used to create this component instance. | string | &#9745; ||
| **instanceName** | The name for this runtime instance of the component.| string | &#9745; ||
| **parameterValues**| Values supplied to override [parameters](./component-schematic.md#parameters) exposed in the ComponentSchematic. | Depends on available parameters of the component spec.||
| **traits**| Additional [workload functionality to attach](./traits.md) to the component instance.| See [traits](./traits.md) documentation.||

[Here's an example](https://github.com/oam-dev/rudr/blob/master/examples/first-app-config.yaml) component definition:

```yaml
# Example component definition in application config
components:
- componentName: helloworld-python-v1
  instanceName: first-app-helloworld-python-v1
  parameterValues:
  - name: target
    value: Rudr
  - name: port
    value: '9999'
  traits:
  - name: ingress
    parameterValues:
    - name: hostname
      value: example.com
    - name: path
      value: /
    - name: service_port
      value: 9999
```

## Traits

For each of your components, you can optionally define one or more traits. A trait represents a piece of add-on functionality that attaches to a component workload, such as traffic routing rules or auto-scaling policies.

**See the [Traits](traits.md) topic guide for more on assigning and managing traits.**

[Here's an example](../../examples/first-app-config.yaml) of an entry to the traits section:

```yaml
# Example trait entry
traits:
- name: ingress
  parameterValues:
  - name: hostname
    value: example.com
  - name: path
    value: /
  - name: service_port
    value: 9999
```

## Scopes

You can deploy one or more of your components within one or more application scopes. A scope represents a logical grouping of components based on common behaviors or dependencies. For example, you might group several component workloads under the same [*health scope*](scopes.md#health-scope) in order to easily probe their aggregate health status, or you might group components together under a common *network scope* to link them to a particular network.

**See the [Scopes](scopes.md) topic guide for more on configuring and applying scopes.**

[Here's an example](../../examples/first-app-config.yaml) of assigning a [pre-configured scope](../../examples/health-scope-config.yaml) (`my-health-scope`) to a given component within the application configuration:

```yaml
# Example scope assignment
applicationScopes:
  - my-health-scope
```
