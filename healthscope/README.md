# Health Scope Controller

Health Scope Controller used for periodically check health scope crd and check the health of all the components related.

## What will health scope controller do?

1. periodically check all component health and update the CR status.
2. serve as a http server, to output aggregated health information.

## How to install?

You could use helm to install it:

```shell script
helm install healthscope ./charts/healthscope
```

## How to use?

After healthscope was installed by helm charts, you could setup an endpoint to it.
By default the charts will use ClusterPort, you could use port mapper to make health scope accessible.

```shell script
export POD_NAME=$(kubectl get pods -l "app.kubernetes.io/name=healthscope,app.kubernetes.io/instance=health" -o jsonpath="{.items[0].metadata.name}")
kubectl port-forward $POD_NAME 8080:80
```

Then you'll be able to visit http://127.0.0.1:8080 to get health scope status.

### Create health scope instance using application configuration

If you want to use health scope, you should create a health scope instance first, using Application Configuration.

A health scope instance is like below:

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: my-health-scope
spec:
  scopes:
    - name: health
      type: core.oam.dev/v1alpha1.HealthScope
      properties:
        - name: probe-method
          value: "kube-get"
        - name: probe-endpoint
          value: ".status"
        - name: probe-timeout
          value: 30
        - name: probe-interval
          value: 60
        - name: failure-rate-threshold
          value: 0
        - name: healthy-rate-threshold
          value: 100.0
        - name: healthThresholdPercentage
          value: 100.0
```

You can also find this example here: [examples/health-scope-config.yaml](../examples/health-scope-config.yaml).

Apply this yaml:

```shell script
$ kubectl apply -f examples/health-scope-config.yaml
applicationconfiguration.core.oam.dev/my-health-scope created
```

Then you will find a health scope instance was created:

```shell script
$ kubectl get health
NAME              AGE
my-health-scope   31s
```

You could get more details from:

```shell script
$ kubectl get health my-health-scope -o yaml
apiVersion: core.oam.dev/v1alpha1
kind: HealthScope
metadata:
  creationTimestamp: "2019-10-20T09:42:30Z"
  generation: 3
  name: my-health-scope
  namespace: default
  ownerReferences:
  - apiVersion: core.oam.dev/v1alpha1
    blockOwnerDeletion: true
    controller: true
    kind: ApplicationConfiguration
    name: my-health-scope
    uid: 1c113398-16f2-4aa1-9dc0-cd05a686d17d
  resourceVersion: "2465881"
  selfLink: /apis/core.oam.dev/v1alpha1/namespaces/default/healthscopes/my-health-scope
  uid: ba3fd01c-9ba6-4e1a-93c0-b96d102700af
spec:
  failureRateThreshold: 0
  healthyRateThreshold: 100
  probeEndpoint: .status
  probeInterval: 60
  probeMethod: kube-get
  probeTimeout: 30
status:
  lastAggregateTimestamp: "2019-10-20T09:43:31.541142387+00:00"
```

Then we could use this health scope in other Application Configuration.

### Add Component to health scope

We add application scopes to our [`first-app-config.yaml`](../examples/first-app-config.yaml).

```shell script
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: first-app
spec:
  components:
    - name: helloworld-python-v1
      instanceName: first-app-helloworld-python-v1
      parameterValues:
        - name: target
          value: Rudr
        - name: port
          value: "9999"
      traits:
        - name: ingress
          parameterValues:
            - name: hostname
              value: example.com
            - name: path
              value: /
            - name: service_port
              value: 9999
+     applicationScopes:
+       - my-health-scope
```

Apply the config file:

```shell script
$ kubectl apply -f examples/first-app-config.yaml
applicationconfiguration.core.oam.dev/first-app created
```

You could check the scope like below:

```shell script
$ kubectl get health -o yaml my-health-scope
  apiVersion: core.oam.dev/v1alpha1
  kind: HealthScope
  metadata:
    creationTimestamp: "2019-10-20T09:42:30Z"
    generation: 10
    name: my-health-scope
    namespace: default
    ownerReferences:
    - apiVersion: core.oam.dev/v1alpha1
      blockOwnerDeletion: true
      controller: true
      kind: ApplicationConfiguration
      name: my-health-scope
      uid: 1c113398-16f2-4aa1-9dc0-cd05a686d17d
    resourceVersion: "2466413"
    selfLink: /apis/core.oam.dev/v1alpha1/namespaces/default/healthscopes/my-health-scope
    uid: ba3fd01c-9ba6-4e1a-93c0-b96d102700af
  spec:
    failureRateThreshold: 0
    healthyRateThreshold: 100
    probeEndpoint: .status
    probeInterval: 60
    probeMethod: kube-get
    probeTimeout: 30
  status:
    components:
    - instanceName: first-app-helloworld-python-v1
      name: helloworld-python-v1
      status: healthy
    lastAggregateTimestamp: "2019-10-20T09:49:22.820141484+00:00"
```

The status indicates that we have successfully added our component to this scope.

### Visit health scope instance to check health

Do you still remember our port mapping in the first step? Visit that url with our health scope instance:
```
$ curl 127.0.0.1:8080/my-health-scope
healthy
```

Then you will find it's healthy now。