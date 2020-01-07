# OpenFaaS Extended Workload

## Install

In this tutorial, we assume you have already successfully installed Rudr. If not, you could see the [Installation Guide](../setup/install.md) to get help.

### Install OpenFaaS Operator with Helm v3

After Rudr was installed, we need to install [OpenFaaS Operator](https://github.com/openfaas-incubator/openfaas-operator), that will be the real workload executor.

```shell script
# create OpenFaaS namespaces
kubectl apply -f https://raw.githubusercontent.com/openfaas/faas-netes/master/namespaces.yml

# add OpenFaaS Helm repo
helm repo add openfaas https://openfaas.github.io/faas-netes/

# generate a random password
PASSWORD=$(head -c 12 /dev/urandom | shasum| cut -d' ' -f1)

kubectl -n openfaas create secret generic basic-auth \
--from-literal=basic-auth-user=admin \
--from-literal=basic-auth-password="$PASSWORD"

# get latest chart version
helm repo update

# install OpenFaaS operator with basic auth enabled
# functionNamespace should be changed to the namespace in which Rudr is installed, in our case, it's default namespace
helm install openfaas openfaas/openfaas \
    --namespace openfaas  \
    --set basic_auth=true \
    --set functionNamespace=default \
    --set operator.create=true
```

If you have trouble connecting with the chart repository, just Clone the [faas-netes](https://github.com/openfaas/faas-netes) repository and install:

```
git clone https://github.com/openfaas/faas-netes.git
helm install openfaas faas-netes/chart/openfaas \
    --namespace openfaas  \
    --set basic_auth=true \
    --set functionNamespace=default \
    --set operator.create=true
```

Finally you could verify the installation by checking the pods.

```shell script
$ kubectl -n openfaas get pods
NAME                                 READY   STATUS    RESTARTS   AGE
alertmanager-67dd8444ff-nn4k8        1/1     Running   0          4m42s
basic-auth-plugin-6f5878475f-ln2k2   1/1     Running   0          4m42s
faas-idler-5c87fbcbdf-tsscj          1/1     Running   2          4m42s
gateway-559fb6f5b8-bsdzq             2/2     Running   0          4m42s
nats-79ffb4bb68-vgzct                1/1     Running   0          4m42s
prometheus-74bc4d6884-rlj2f          1/1     Running   0          4m42s
queue-worker-55fd5749d-7q678         1/1     Running   1          4m42s
```

Fetch your public IP or NodePort via `kubectl get svc -n openfaas gateway-external -o wide`.

If using a remote cluster, you can port-forward the gateway to your local machine:

```shell script
kubectl port-forward -n openfaas svc/gateway 31112:8080 &
```

Then you could visit the OpenFaaS web by `http://127.0.0.1:31112`.

User name is `admin`, and you could get the password by:

```shell script
$ echo $PASSWORD
1b26c16c5af2343ee82be46635c0b44f7457a176
```


## Create Component with OpenFaaS Workload

An OpenFaaS workload example is like below:

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ComponentSchematic
metadata:
  name: openfaas
spec:
  workloadType: openfaas.com/v1alpha2.Function
  osType: linux
  parameters:
    - name: image
      type: string
      default: alexellisuk/go-echo:0.2.2
    - name: handler
      type: string
      default: ""
    - name: write_debug
      type: string
      default: "true"
  workloadSettings:
    - name: image
      type: string
      description: docker image name of this function
      fromParam: image
      value: alexellisuk/go-echo:0.2.2
      required: true
    - name: handler
      type: string
      description: entrypoint of your function, eg. node main.js
      required: false
      fromParam: handler
    - name: environment
      type: array
      description: environments of functions
      value:
        - name: write_debug
          type: string
          value: "true"
          fromParam: write_debug
```

Note that, the workload type is `openfaas.com/v1alpha2.Function`.

Explanation of OpenFaaS workload configurations in detail:

1. `image`: image is the docker image name of function, the type of image is string, this field is always required for OpenFaaS workload. You could also make this field read from parameters.
2. `handler`: handler is the entry point of your function, the type is string, this field is optional, and also could read from parameters.
3. `environment`: the type of environment is array, and you should write the environment list in the field value. Each environment could be read from parameters.

## Create Application Configuration to instantiate the Component

Now we should create an Application Configuration to instantiate the OpenFaaS Component.

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: openfaas
spec:
  components:
    - componentName: openfaas
      instanceName: gofast
      parameterValues:
        - name: image
          value: functions/nodeinfo
        - name: handler
          value: "node main.js"
        - name: write_debug
          value: "false"
```

Note that we just used `parameterValues` to override parameters in the previous OpenFaaS Component.

> Check the sample YAML files above in [openfaasapp.yaml](../../examples/openfaasapp.yaml).

We could deploy it by:

```shell script
$ kubectl apply -f examples/openfaasapp.yaml
componentschematic.core.oam.dev/openfaas configured
applicationconfiguration.core.oam.dev/openfaas created
``` 

Then you could find functions have been created:

```shell script
$ kubectl get functions
NAME       AGE
nodeinfo   27s
```

You could also visit the function deployed:

```shell script
$ curl http://127.0.0.1:31112/function/nodeinfo
Hostname: nodeinfo-86b679f76f-sgkbb

Platform: linux
Arch: x64
CPU count: 2
Uptime: 796748
```

## Apply trait

Manual Scaler trait could also be applied to OpenFaaS workload like below:

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: openfaas
spec:
  components:
    - componentName: openfaas
      instanceName: nodeinfo
      parameterValues:
        - name: image
          value: functions/nodeinfo
        - name: handler
          value: "node main.js"
        - name: write_debug
          value: "false"
      traits:
        - name: manual-scaler.core.oam.dev/v1alpha1
          properties:
            replicaCount: 2
```

It will set OpenFaaS workload to have at least 2 replicas. After change the Application Configuration yaml file, apply it.

```
$ kubeclt apply -f examples/openfaasapp.yaml
componentschematic.core.oam.dev/openfaas unchanged
applicationconfiguration.core.oam.dev/openfaas configured
```

Then we could see the `com.openfaas.scale.min` label of the function has changed.

```shell script
$ kubectl get functions nodeinfo -o yaml
apiVersion: openfaas.com/v1alpha2
kind: Function
metadata:
  name: nodeinfo
  namespace: default
  labels:
    com.openfaas.scale.min: "2"
spec:
  environment:
    write_debug: "false"
  handler: node main.js
  image: functions/nodeinfo
  name: nodeinfo
```

If you want to change the minimal replica to `3`, just change the application configuration yaml.

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: openfaas
spec:
  components:
    - componentName: openfaas
      instanceName: nodeinfo
      parameterValues:
        - name: image
          value: functions/nodeinfo
        - name: handler
          value: "node main.js"
        - name: write_debug
          value: "false"
      traits:
        - name: manual-scaler.core.oam.dev/v1alpha1
          properties:
            replicaCount: 3
```

Apply it again.

```
$ kubeclt apply -f examples/openfaasapp.yaml
componentschematic.core.oam.dev/openfaas unchanged
applicationconfiguration.core.oam.dev/openfaas configured
```

Then you could see the `com.openfaas.scale.min` label has been changed.

```shell script
$ kubectl get functions nodeinfo -o yaml
apiVersion: openfaas.com/v1alpha2
kind: Function
metadata:
  name: nodeinfo
  namespace: default
  labels:
    com.openfaas.scale.min: "3"
spec:
  environment:
    write_debug: "false"
  handler: node main.js
  image: functions/nodeinfo
  name: nodeinfo
```

In real world practices, the OpenFaaS `Component` is expected to be defined by Devs, while `ApplicationConfiguration` and manual-scaler `Trait` be managed by App Ops.
You could check further practices guideline in [roles and responsibilities](https://github.com/oam-dev/spec/blob/master/2.overview_and_terminology.md#roles-and-responsibilities) explanation of OAM spec.