# Using Scylla

This guide explains the basics of using Scylla to install applications on your Kubernetes cluster. It assumes that you have already [installed Scylla](install.md).

If you are simply interested in having a quick try, you may wish to begin with the [Quickstart Guide](quickstart.md). This chapter covers how to use Scylla and install one example application.

## Four Concepts with One Action

Scylla is a project implements the [Hydra specification](https://github.com/microsoft/hydra-spec) for Kubernetes. So before using Scylla, you may need to understand some concepts of Hydra. If you're interested to learn more, you can directly read the [Hydra specification](https://github.com/microsoft/hydra-spec). 

### Component schematics
 
The component schematics is where developers declare the operational characteristics of the code they deliver in infrastructure neutral terms.

In Scylla, this is a CRD in Kubernetes, the schema of component schematics is something like below:

```yaml
apiVersion: core.hydra.io/v1alpha1
kind: ComponentSchematic
metadata:
  name: nginx-replicated
spec:
  workloadType: core.hydra.io/v1alpha1.ReplicatedService
  os: linux
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

1. workload type used to declare which kind of workload pattern you will use, we will explain it in the following content.
2. os and arch are used to specify which runtime the component can run. These two values can be omitted, and default os is linux while default arch is amd64.
3. containers are almost the same with kubernetes container spec, some differences are we allow to bind config here, like the example use. The configs in container are implemented using Kubernetes ConfigMap in Scylla. We will bind to config as a volume to the pod.
4. parameters can be used as reference values in container spec, such as environment values.

The only field we missed in the example is workload settings, this field is used to declare values for non-container settings that should be passed to the workload runtime.

So if we want to deploy an application, we need to deploy component first. But the component won't run, it's just like a container image, we have to build the image before run container.

Use command `kubectl apply -f <component.yaml>` to make the component available.

You can run command `kubectl get components` to find what are the components prepared ready. 

Use `kubectl get component <component-name> -o yaml` to find details of the component.

### Workload types

A workload type is an indicator to the runtime as to how it should execute the given workload. In other words, it provides a single field by which the developer can indicate to the runtime how the developer intends for this component to be executed.

The Hydra Spec defines two kinds of workload types:

* Core workload types
* Extended workload types

Now Scylla only has all of the six core workload types, you can find more details in [workloads documentation](workloads.md).

So workload types don't have any CRDs, it's just one of the fields in component. Platform users can't define custom workload types, 
they can only choose workload types predefined by the platform runtime.  

### Traits

A trait is a discretionary runtime overlay that augments a component workload type (workloadType) with additional features. 
It is an opportunity for those in the application operator role to make specific decisions about the configuration of components, but without involving the developer. 

A trait can be any operational configuration of a distributed application that applies to an individual component, such as traffic routing rules 
(e.g., load balancing policy, network ingress routing, circuit breaking, rate limiting), auto-scaling policies, upgrade strategies, and more.

So trait is another concept like component, it has CRDs predefined by the platform operator which application operator can choose to use.

Now we only have three traits, and will add more in the near future.

You can run command `kubectl get traits` to find what are the traits supported by the platform. 

Use `kubectl get trait <trait-name> -o yaml` to find schema detail of the trait.
 
You can find more information in [traits documentation](traits.md).

### Application scopes

Application scopes are used to group components together into logical applications by providing different forms of application boundaries with common group behaviors.

Scylla will soon implement health and network application scope, now we don't have any application scopes implemented.

Application scopes are also concept like component and trait. 

### Operational configuration

Operational configuration defines a deployment of components, their traits, and application scopes. 
So this is the **action** who will make the real application run in Kubernetes.

The configuration schema is like below:

```yaml
apiVersion: core.hydra.io/v1alpha1
kind: OperationalConfiguration
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

It will contain components, traits and application scopes, while you can see in the example that we don;t use application scope here.
The traits are also optional. At least one component is required.

The component, traits and application scopes here are all references. So you must confirm the platform has these three kind of resources what you defined in the configuration file.
    
