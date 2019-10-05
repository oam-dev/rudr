# Traits

A trait is a discretionary runtime overlay that augments a component workload type (`workloadType`) with additional features.

In terms of implementation details, traits are a kind of CRD defined by Scylla. The specification of trait defines two major things:

1. The list of workload types to which this trait applies.
2. The list of configurable parameters on this trait.

In Scylla we have three traits now. They are introduced one by one below. 

## Autoscaler Trait

Autoscaler trait is used autoscale components with replicable workloads. This is implemented by the Kubernetes [Horizontal Pod Autoscaler](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/).

You can find more details about the autoscaler trait in the [charts templates directory](../charts/scylla/templates/traits.yaml), 
or you can use `kubectl get traits autoscaler -o yaml` to get details from your cluster if you have installed Scylla.

Now autoscaler trait can be used on components with Service and Task workload types.

We currently support four configurable fields in autoscaler trait:

* minimum: This represents the lower limit for the number of replicas to which the autoscaler can scale down. It defaults to 1 component.
* maximum: This represents the upper limit for the number of replicas to which the autoscaler can scale up. It cannot be less that minReplicas.
* memory: This represents the memory consumption threshold (as percent) that will cause a scale.
* cpu: This represents the CPU consumption threshold (as percent) that will cause a scale.


## Ingress Trait

Ingress trait is used for ccomponents with service workloads. It can provide load balancing, SSL termination and name-based virtual hosting.

You can find more details about the ingress trait in the [charts templates directory](../charts/scylla/templates/traits.yaml), 
or you can use `kubectl get traits ingress -o yaml` to get from your cluster if you have installed Scylla.

Currently the ingress trait can be used on components with Service and SingletonService workload types.

Currently, three configurable fields are supported for the ingress trait: 

* hostname: This represents the host name for the ingress.
* service_port: This represents the port number on the service which will be bind to the ingress.
* path: This represents the path to expose by the ingress. Default is '/'.


## Manual Scaler Trait

Manual Scaler trait is used to manually scale components with replicable workload types.

You can find further details about the manual scaler trait in the [charts templates directory](../charts/scylla/templates/traits.yaml), 
or you can use `kubectl get traits manual-scaler -o yaml` to get information from your cluster if you have installed Scylla.

Currently the Manual Scaler trait can be used on Components with Service and Task workload types.

Currently we support only one field for the Manual Scaler trait:

* replicaCount: This represents the number of replicas the workload is expected to run at all times. 
