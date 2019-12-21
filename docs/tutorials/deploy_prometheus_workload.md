# Prometheus Extended Workload

## Deploy Prometheus Operator

```shell script
$ kubectl apply -f https://raw.githubusercontent.com/coreos/prometheus-operator/master/bundle.yaml
```

## Define the Prometheus component

The component must have a workloadType combined with `GROUP/VERSION.KIND`, so the Non-Intrusive Workload will find which custom resource to create.

Then put the whole spec in the workloadSettings value with a name called `spec` like below.

```
apiVersion: core.oam.dev/v1alpha1
kind: ComponentSchematic
metadata:
  name: prometheus
spec:
  workloadType: monitoring.coreos.com/v1.Prometheus
  osType: linux
  workloadSettings:
    - name: spec
      type: object
      description: the spec of prometheus-operator
      required: true
      value:
        serviceAccountName: default
        serviceMonitorSelector:
          matchLabels:
            team: frontend
        resources:
          requests:
            memory: 400Mi
        enableAdminAPI: true
```

## Prepare the application configuration

The application configuration just need to use this component. 

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: prometheus
spec:
  components:
    - componentName: prometheus
      instanceName: prometheus-app
```

## Apply our configurations

```shell script
$ kubectl apply -f examples/prometheusapp.yaml
componentschematic.core.oam.dev/prometheus created
applicationconfiguration.core.oam.dev/prometheus created
```

we could see that an Prometheus Operator CR was created by rudr.

```shell script
$ kubectl get prometheuses
NAME             AGE
prometheus-app   37s
```

Then the Prometheus operator we create an real Prometheus app described by the CR.

```shell script
$ kubectl get statefulset
NAME                        READY   AGE
prometheus-prometheus-app   1/1     6m21s
```

You could change the component spec as you like if you want.

## TODO

- [ ] Register and generate serving information for this Prometheus workload for other workload to consume. 