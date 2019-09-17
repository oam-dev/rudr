# Traits

A trait is a discretionary runtime overlay that augments a component workload type (`workloadType`) with additional features.

Trait is a kind of CRD defined by Scylla, the specification of trait defines two major things:
The list of workload types to which this trait applies, and the list of configurable parameters on this trait.

In Scylla we have three traits now, let me introduce them one by one.

## Autoscaler

Autoscaler trait is used for replicable workload to auto scale. This is implement by Kubernetes [Horizontal Pod Autoscaler](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/).

You can find the detail autoscaler trait in the [charts templates directory](../charts/scylla/templates/traits.yaml), 
or you can use `kubectl get traits autoscaler -o yaml` to get from your cluster if you have installed Scylla.

Now autoscaler trait can be used to Service and Task workload types.

Now we support four field in autoscaler trait:

* minimum: This represents the lower limit for the number of replicas to which the autoscaler can scale down. It defaults to 1 pod.
* maximum: This represents the upper limit for the number of replicas to which the autoscaler can scale up. It cannot be less that minReplicas.
* memory: This represents the memory consumption threshold (as percent) that will cause a scale.
* cpu: This represents the CPU consumption threshold (as percent) that will cause a scale.


## Ingress

Ingress trait is used for service kind of workloads, it can provide load balancing, SSL termination and name-based virtual hosting.

You can find the detail ingress trait in the [charts templates directory](../charts/scylla/templates/traits.yaml), 
or you can use `kubectl get traits ingress -o yaml` to get from your cluster if you have installed Scylla.

Now ingress trait can be used to Service and SingletonService workload types.

Now we support three field in ingress trait:

* hostname: This represents the host name for the ingress.
* service_port: This represents the port number on the service which will be bind to the ingress.
* path: This represents the path to expose by the ingress. Default is '/'.


## Manual Scaler

Manual Scaler trait is used for replicable workload to scale manually.

You can find the detail manual scaler trait in the [charts templates directory](../charts/scylla/templates/traits.yaml), 
or you can use `kubectl get traits manual-scaler -o yaml` to get from your cluster if you have installed Scylla.

Now Manual Scaler trait can be used to Service and Task workload types.

Now we support only one field in Manual Scaler trait:

* replicaCount: This represents the number of replicas the workload is expected. 