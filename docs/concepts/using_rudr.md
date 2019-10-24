# Using Rudr

This guide explains the basics of using Rudr to install applications on your Kubernetes cluster. It assumes that you have already [installed Rudr](../setup/install.md).

If you'd rather jump right in and start using Rudr, check out the [Rudr Tutorial](../tutorials/deploy_and_update.md), which walks you through installing, inspecting, and updating an application and its constituent components.

## Four Concepts with One Action

Rudr is a reference implementation of the [Open Application Model specification](https://github.com/oam-dev/spec) for Kubernetes. So before using Rudr, you may need to understand some concepts of Open Application Model. If you're interested to learn more, you can directly read the [Open Application Model specification](https://github.com/oam-dev/spec). 

### Component schematics
 
The component schematics is where developers declare the operational characteristics of the code they deliver in infrastructure neutral terms.

In Rudr, the schema of component schematics is something like below:

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ComponentSchematic
metadata:
  name: nginx-replicated
spec:
  workloadType: core.oam.dev/v1alpha1.Server
  osType: linux
  arch: amd64
  containers:
    - name: server
      image: nginx:latest
      config:
        - name: "/etc/access/default_user.txt"
        - value: "admin"
      ports:
        - name: http
          containerPort: 80
          protocol: TCP
  parameters:
    - name: poet
      type: string
      default: Yeats
```

In the example, we can see that there are five fields in spec:

1. workload type used to declare which kind of workload pattern you will use; we will explain this further in the following content.
2. os and arch are used to specify which runtime the component can run. These two values can be omitted causing the os to default to linux and the arch to default to amd64.
3. containers are almost the same with kubernetes container spec. However, some differences are we allow to bind the config here. The configs in container are implemented using the Kubernetes ConfigMap in Rudr. In this example, we will bind to config as a volume to the pod.
4. parameters can be used as reference values in container spec, such as environment values.

The only field we missed in the example is workload settings. This field is used to declare values for non-container settings that should be passed to the workload runtime.

So if we want to deploy an application, we need to deploy component first. However, just like you need a container image before you can run a container, a component will not run on its own but is necessary to deploy before running an application.

Use command `kubectl apply -f <component.yaml>` to make the component available.

You can run command `kubectl get components` to find what are the components prepared ready. 

Use `kubectl get component <component-name> -o yaml` to find details of the component.

### Workload types

A workload type is an indicator to the runtime as to how it should execute the given workload. In other words, it provides a single field by which the developer can indicate to the runtime how they intend for this component to be executed.

The Open Application Model Spec defines two broad categories of workloads:

* Core workload types
* Extended workload types

Within these these two categories of workloads, there are several workload types. A non-exhaustive list of workload types might include a server, singleton server, and task. Currently Rudr only has all of the six core workload types, you can find more details in [workloads documentation](workloads.md).

It's important to understand that workload types don't have any CRDs. They are simply just a field within a component. Platform users can't define custom workload types. 
They can only choose workload types predefined by the platform runtime.  

### Traits

A trait is a discretionary runtime overlay that augments a component workload type (workloadType) with additional operational features. 
It is an opportunity for those in the application operator role to make specific decisions about the configuration of components, without depending on the involvement of the developer. 

A trait can be any application configuration of a distributed application that applies to an individual component, such as traffic routing rules 
(e.g., load balancing policy, network ingress routing, circuit breaking, rate limiting), auto-scaling policies, upgrade strategies, and more.

Traits can be selected by the application operator as they are implemented as CRDs predefined by the platform operator.

Currently we only have several traits but have plans add more in the near future. We encourage submitting PRs on Open Application Model if you feel like you have a good idea for a trait that could be broadly used by others. 

`kubectl get traits` returns the traits the platform supports. 

Use `kubectl get trait <trait-name> -o yaml` to find schema detail of the trait.
 
You can find more information in [traits documentation](../concepts/traits.md).

### Application scopes

Application scopes are used to group components together into logical units that are bound by a common dependency. An example of this is a network scope. Even if you deploy 5 components within an application configuration, perhaps you want three of those components deployed in one network and the other two deployed in a separate network. To achieve this behavior, you could tag the former with network scope A  and the later with network scope B in the application configuration.

Rudr will soon implement health and network application scopes; however currently we don't have any application scopes implemented.

Application scopes are also concept like component and trait. 

### Application configuration

Application configuration defines a deployment of components, their traits, and application scopes. 
So this is the **action** that will make the real application run in Kubernetes.

The configuration schema is like below:

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: first-app
spec:
  components:
  - name: nginx-replicated
    instanceName: first-app-nginx
    parameterValues:
      - name: poet
        value: Eliot
    traits:
      - name: manual-scaler
        parameterValues:
          - name: replicaCount
            value: 1
```

While you can't see an implementation of scopes here in this example, in future Rudr versions it will contain components, traits and application scopes.
The traits are also optional. The only hard requirement is that at least one component or application scope is required.

The component, traits and application scopes here are all references. You must confirm that the platform has these three kind of resources that you defined in the configuration file.
    
