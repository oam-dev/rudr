# Rudr documentation

*Rudr* is the Kubernetes reference implementation of the [Open Application Model](https://github.com/oam-dev/spec) (OAM) specification, a team-centric standard for building cloud-native apps, where:

<html>
<table style="border:none;">
<tr>
<td><img src="./media/developer-role.png" /></td>
<td><b>Developers</b> define application <a href="#component-schematic" style="color:rgb(121, 149, 64);font-weight:bold;text-decoration:none">components</a>,</td>
</tr>
<tr>
<td><img src="./media/app-operator-role.png" /></td>
<td><b>Application operators</b> create instances of those components and assign them to <a href="#application-configuration" style="color:rgb(49, 133, 156);font-weight:bold;text-decoration:none">application configurations</a>, and</td>
</tr>
<tr>
<td><img src="./media/infra-operator-role.png" /></td>
<td><b>Infrastructure operators</b> declare, install, and maintain the <a href="#application-configuration" style="color:rgb(127, 101, 159);font-weight:bold;text-decoration:none">underlying services</a> available on the platform.</td>
</tr>
</table>
</html>



## [Install Rudr](./setup/install.md)
Install the Rudr runtime and its dependencies.

## [Tutorial](./tutorials/deploy_and_update.md)
Learn how to deploy, inspect, and update a Rudr application.

## Concepts
Learn more about the main application model constructs: components (and their workloads), traits, and application configurations.

### [Component Schematic](./concepts/component-schematic.md)

Learn how a <span style="color:rgb(121, 149, 64);font-weight:bold;">developer</span> can define the functional units that may be instantiated as part of a larger distributed application and their respective [workloads](./concepts/workloads.md).

### [Application Configuration](./concepts/application-configuration.md)

Learn how an <span style="color:rgb(49, 133, 156);font-weight:bold">application operator</span> can define how an overall application will be instantiated and configured.

### [Traits](./concepts/traits.md)

Learn how an <span style="color:rgb(49, 133, 156);font-weight:bold">application operator</span> can attach operational features to component workloads of an application.

### [Scopes](./concepts/scopes.md)
Learn how an <span style="color:rgb(49, 133, 156);font-weight:bold">application operator</span> can define application boundaries by grouping components with common properties and dependencies.

## How-To's

### [Create a component from scratch](how-to/create_component_from_scratch.md)

Build a component from source code to use for testing.

### [Manage OAM files with Helm/Kustomize](how-to/using_helm_kustomize_manage_oam.md)

Learn how to use Helm/Kustomize tools to manage your OAM .yaml filese.

### [Migrate existing Kubernetes resources](./how-to/migrating.md)

Here are tips and best practices for migrating exsiting Kubernetes applications to use Rudr.

## Develop

#### [Writing a Trait](./developer/writing_a_trait.md)

Explain how to write a trait for Rudr.

#### [Debug](./developer/debug.md)

How to debug when using Rudr.

## [FAQ](./faq.md)

Find answers to commonly asked questions about Rudr and the Open Application Model (OAM).
