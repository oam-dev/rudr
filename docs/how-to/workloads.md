# Workloads

Now Scylla has all the core workload type, they are as belows: 

|Name|Type|Service endpoint|Replicable|Daemonized|
|-|-|-|-|-|
|Server|core.hydra.io/v1alpha1.Server|Yes|Yes|Yes
|Singleton Server|core.hydra.io/v1alpha1.SingletonServer|Yes|No|Yes
|Task|core.hydra.io/v1alpha1.Task|No|Yes|No
|Singleton Task|core.hydra.io/v1alpha1.SingletonTask|No|No|No
|Worker|core.hydra.io/v1alpha1.Worker|No|Yes|Yes
|Singleton Worker|core.hydra.io/v1alpha1.SingletonWorker|No|No|Yes

## Server

A Server is used for long-running, scalable workload that have a network endpoint with a stable name to receive network traffic for the component as a whole. 
Common use cases include web applications and services that expose APIs.

The Server workload in Scylla is implemented by a [Kubernetes Deployment](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/) binding with a [Kubernetes Service](https://kubernetes.io/docs/concepts/services-networking/service/).

So you can use Ingress, Autoscaler or Manual Scaler trait binding with it in a configuration.

## Singleton Server

A Singleton Server is a special kind of Server, just like the name pointed out, the only difference is this is a singleton.

The Singleton Server in Scylla is implemented by a [Kubernetes Deployment](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/) binding with a [Kubernetes Service](https://kubernetes.io/docs/concepts/services-networking/service/).

> Operators should not attempt to modify the `replicaCount` on a `Deployment` created by a `SingletonServer`.

This is a singleton, so you can't use Autoscaler or Manual Scaler trait.

Of course binding an Ingress trait is OK just like the Server Workload. 

## Task

A Task is used to run code or a script to completion. Commonly used to run cron jobs or one-time highly parallelizable tasks that exit and free up resources upon completion. 

The Task in Scylla is implemented by a [Kubernetes Job](https://kubernetes.io/docs/concepts/workloads/controllers/jobs-run-to-completion/), you can use  Autoscaler or Manual Scaler trait with it.

## Singleton Task

A singleton task is a special case of the task workload type that is limited to at most one replica. 

It is also implemented by a [Kubernetes Job](https://kubernetes.io/docs/concepts/workloads/controllers/jobs-run-to-completion/).

This is a singleton, so you can't use Autoscaler or Manual Scaler trait.

## Worker

A worker is used for long-running, scalable workloads that do not have a service endpoint for network requests, aside from optional liveliness and readiness probe endpoints on individual replicas. Workers are typically used to pull from queues or provide other offline processing. 

## Singleton Worker

A singleton worker is a special case of the worker workload type that is limited to at most one replica. 
