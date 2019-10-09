# Migrating Kubernetes Resources to Rudr

This document explains, at a high level, how to represent your Kubernetes applications using the Open Application Model specification.

## Terms

The terms here are described elsewhere, particularly in the [specification](https://github.com/microsoft/hydra-spec). They are summarized here for convenience.

- Component: A component describes a particular workload (or microservice) that can be deployed.
- Component Instance: A particular instance of a component. If one Component is deployed in two Application Configurations, it will create two Component Instances, each of which is managed by its Application Configuration
- Workload type: Open Application Model describes the basic behavior of a Component using workload types. There are 6 core workload types:
  - SingletonServer: This service listens on a network interface. Only one instance of the component's pod can run at a time.
  - Server: This service listens on a network interface, but multiple replicas of the component's pod can be running concurrently.
  - SingletonTask: This component only runs for a short period of time (it is not a daemon). It does not listen on a network interface. And at any given time, only one of these may run per application.
  - Task: This component only runs for a short period of time (it is not a daemon). It does not listen on a network interface. Many replicas of this task may run concurrently.
  - SingletonWorker: This component is long-running, but does not listen on a network interface. Only one pod can be running per application.
  - Worker: This component is long-running, but does not listen on a network interface. Multiple workers may run concurrently.
- Application Configuration: This resource describes an application as a list of components. Configuration information (parameter values) may be passed into components via the application configuration. Traits and scopes are attached to components in the Application Configuration
- Traits: A trait describes an operational behavior that should be attached to a component at runtime. For example, a Server may have an Autoscaler trait attached.
- Scopes: A scope is an arbitrary group of Component Instances that share a behavior. For example, the health check scope facilitates an aggregate health check of all Component Instances in that Scope. If any Component Instance's health check fails, the Scope's health check fails.

## Separation of Concerns

One of the questions we occasionally hear from seasoned Kubernetes developers is _What do I gain from Open Application Model?_. To be clear, part of the design of Open Application Model was to make it easier for developers to work with Kubernetes without needing to understand the operational aspects of Kubernetes. But the real virtue we see in Open Application Model is its separation of concerns.

Today's Kubernetes objects are built to _describe a concept_. A Deployment describes a replicable service that can have certain rollout strategies applied.

But Open Application Model attempts to start with a different premise, and then describe things accordingly:

> Cloud Native Applications have responsibilities described by three different roles: Developers, Application Operators, and Infrastructure Operators.

Each of these roles has a different job, and different concerns. A developer is responsible for producing a runnable workload. An application operator is responsible for executing an application inside of Kubernetes. An infrastructure operator is responsible for configuring and running Kubernetes itself.

In our view, the developer is responsible for creating and maintaining Components. The application operator takes responsibility for the Application Configuration. And the infrastructure operator runs Rudr, and decides how Open Application Model's Traits and Scopes are used in practice. And Rudr's job is to take these various inputs and transform them into underlying Kubernetes types.

## Workflow

Traits and Scopes are provided by Rudr. You can list them with `kubectl get traits` and `kubectl get scopes`.

You may install your own components.

To create a new Rudr application, create an `ApplicationConfiguration` YAML file and specify the components, traits, scopes, and parameters.

Rudr watches for new Application Configurations. When one is created, Rudr will read it, load the component, trait, and scope definitions, and then create new resources in Kubernetes.

Likewise, when Application Configurations are modified or deleted, Rudr will respond to those events, managing the resources for those components, scopes, and traits.

## Converting a Kubernetes Application to Rudr

A major goal of the Open Application Model specification is to separate operational concerns from developer concerns. So a `ComponentSchematic` describes a component from a developer's view, while an `ApplicationConfiguration` creates instances of Components, and attaches configuration data to them.

To convert a Kubernetes application to Rudr, you can follow these steps:

1. Describe your workloads (microservices) as Components.
    - A `Deployment` or `ReplicaSet` can be converted to one of `SingletonServer`, `Server`, `SingletonWorker`, or `ReplicableWorker` depending on its runtime requirements
    - A `Job` can be converted to one of `SingletonTask` or `ReplicableTask`
    - A `Pod` can be converted to `SingletonServer` or `SingletonWorker`
    - At this time, `StatefulSets` and `DaemonSets` do not have Open Application Model equivalents.
    - Expose parameters. For example, if your container image takes `FOO` as an environment variable, you may expose a parameter that allows an operator to set `FOO`'s value.
2. Create an `ApplicationConfiguration` YAML file
3. Compose your application by listing which Components (defined above) should be instantiated as part of your application.
    - For each component...
        1. Determine if any Traits need to be applied
            - `Ingress` and `HorizontalPodAutoscaler` have direct equivalents in Rudr, using the `Ingress` and `Autoscaler` traits
            - Use `kubectl get traits` to see what other traits may apply
        2. Determine what Scopes need to be applied
            - Use `kubectl get scopes` to see what scopes may apply
        3. Set parameter values
            - Look at the component's parameters and see if you need to set or override values for any of these

At the end of this process, you should have one or more `ComponentSchematic` definitions and an `ApplicationConfiguration` definition. You can put these in lots of files, or put them all in one file. But the `ComponentSchematic` needs to be loaded to the Kubernetes API server before an `ApplicationConfiguration` can reference it.
