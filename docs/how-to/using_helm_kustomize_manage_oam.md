# Using Helm/Kustomize to manage OAM yamls

OAM use parameters/variables/properties which could be very long and complicated.
In this tutorial, we will introduce using tools (such as helm, kustomize) to solve this problem.

## Using Helm

We have introduced how to [create a helloworld-python component from scratch](./create_component_from_scratch.md) and [deploy it using Application Configuration](../tutorials/deploy_and_update.md).

You could find the [hello-rudr](../../examples/charts/hello-rudr) in `examples` folder, this is the same `hellowworld-python`app built by helm chart.

If you have installed rudr, you could easily install this hello-rudr chart by helm v3. 

Assume you have lots of parameters in appconfig, you could just use helm templates and make them configurable by values.

In this example, we mainly make `target`, `port` configurable.

```yaml
kind: ApplicationConfiguration
apiVersion: core.oam.dev/v1alpha1
metadata:
  name: "{{ .Release.Name }}"
spec:
  components:
    - componentName: "{{ .Release.Name }}-{{ .Values.appVersion}}"
      instanceName: "{{ .Release.Name }}-{{ .Values.appVersion}}"
      parameterValues:
        - name: target
          value: "{{ .Values.target }}"
        - name: port
          value: "{{ .Values.port }}"
      traits:
        - name: ingress.core.oam.dev/v1alpha1
          properties:
            hostname: example.com
            path: /
            servicePort: {{ .Values.port }}
        - name: manual-scaler.core.oam.dev/v1alpha1
          properties:
            replicaCount: {{ .Values.replicaCount }}
```

So they could be configured in `values.yaml` like below:

```yaml
# Default values for hello-rudr.
# This is a YAML-formatted file.
# Declare variables to be passed into your templates.

replicaCount: 1
appVersion: v1
target: Rudr
port: "9999"

```

The `values.yaml` file is clean and easy to understand. 

You could even classify parameters to make them more clear, for example:

 ```yaml

appVersion: v1

scale:
  replicaCount: 1

service:
  target: Rudr
  port: "9999"
 
 ```

Of cource, you should change your templates to:

```yaml
kind: ApplicationConfiguration
apiVersion: core.oam.dev/v1alpha1
metadata:
  name: "{{ .Release.Name }}"
spec:
  components:
    - componentName: "{{ .Release.Name }}-{{ .Values.appVersion}}"
      instanceName: "{{ .Release.Name }}-{{ .Values.appVersion}}"
      parameterValues:
        - name: target
          value: "{{ .Values.service.target }}"
        - name: port
          value: "{{ .Values.service.port }}"
      traits:
        - name: ingress.core.oam.dev/v1alpha1
          properties:
            hostname: example.com
            path: /
            servicePort: {{ .Values.port }}
        - name: manual-scaler.core.oam.dev/v1alpha1
          properties:
            replicaCount: {{ .Values.replicaCount }}
```

## Using Kustomize

[Kustomize](https://github.com/kubernetes-sigs/kustomize) is a tool to customize kubernetes YAML configurations.
Using Kustomize is a little different, the way kustomize used is just like a patch.

We also use [hello-rudr example](../../examples/kustomize/hello-rudr) as an example with the `helloworld-python` app.

First we put the original yaml in [`base`](../../examples/kustomize/hello-rudr/base) directory,
then we define our patch in [`overlay`](../../examples/kustomize/hello-rudr/overlay/production) like below:

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: first-app
spec:
  components:
    - componentName: helloworld-python-v1
      instanceName: patched-app
      parameterValues:
        - name: target
          value: Hello
        - name: port
          value: "8888"
```

In this example, we override 3 parameters here:

1. instanceName: change from `first-app-helloworld-python-v1` to `patched-app`
2. parameterValues(target): change from `Rudr` to `hello`
3. parameterValues(port): change from `9999` to `8888`

Finally, you could use `kustomize build` to see the result:

```console
$ kustomize build overlay/production
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  labels:
    variant: production
  name: production-first-app
spec:
  components:
  - componentName: helloworld-python-v1
    instanceName: patched-app
    parameterValues:
    - name: target
      value: Hello
    - name: port
      value: "8888"
---
apiVersion: core.oam.dev/v1alpha1
kind: ComponentSchematic
metadata:
  labels:
    variant: production
  name: production-helloworld-python-v1
spec:
  containers:
  - env:
    - fromParam: target
      name: TARGET
    - fromParam: port
      name: PORT
    image: oamdev/helloworld-python:v1
    name: foo
    ports:
    - containerPort: 9999
      name: http
      protocol: TCP
  name: helloworld-python
  parameters:
  - default: World
    name: target
    type: string
  - default: "9999"
    name: port
    type: string
  workloadType: core.oam.dev/v1alpha1.Server
```

You could apply the result by:

```shell script
kustomize build overlay/production | kubectl apply -f -
```

So next time you just need to change the `patch.yaml` to make things easier.