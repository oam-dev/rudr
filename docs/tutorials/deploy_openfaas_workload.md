# OpenFaaS extended workload

## Install

In this tutorial, we assume you have already successfully installed rudr. If not, you could see the [Installation Guide](../setup/install.md) to get help.

After rudr installed, we still need to install [OpenFaaS Operator](https://github.com/openfaas-incubator/openfaas-operator), that will be the real workload executor.

The installation steps for OpenFaaS Operator are almost the same with the guides in the [OpenFaaS Operator README docs](https://github.com/openfaas-incubator/openfaas-operator#deploy-openfaas-with-the-operator).

The only difference you should care about is that the functionNames should use the same namespace you have installed rudr. The namespace is `default` in rudr's helm chart, if you haven't manually changed it, we should use `default` here.    

### Install OpenFaaS Operator with Helm v3

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

# install with basic auth enabled
helm install openfaas openfaas/openfaas \
    --namespace openfaas  \
    --set basic_auth=true \
    --set functionNamespace=default \
    --set operator.create=true
```

If you have trouble connecting with the chart repository, just Clone the [faas-netes repository](https://github.com/openfaas/faas-netes) and install:

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

Now rudr support three main field of the OpenFaaS workload, they are `image`, `handler` and `environment`. An example is like below:

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
      default: alexellis/gofast023
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
      value: alexellis/gofast023
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

Firstly, we must set the workloadType as `openfaas.com/v1alpha2.Function`, then rudr will understand this component should work with OpenFaaS workload.

Then, we should fill the workloadSettings properly:

1. `image`: image is the docker image name of function, the type of image is string, this field is always required for OpenFaaS workload. You could also make this field read from parameters.
2. `handler`: handler is the entry point of your function, the type is string, this field is optional, and also could read from parameters.
3. `environment`: the type of environment is array, and you should write the environment list in the field value. Each environment could be read from parameters.

## Create Application Configuration to use the Component

After create the component, we should create an application configuration to use it.

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

Yes, just set the `parameterValues` to override parameters in component.

We have write these two example yaml files in [openfaasapp.yaml](../../examples/openfaasapp.yaml).

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
