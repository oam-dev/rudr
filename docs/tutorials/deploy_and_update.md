# Tutorial: Deploy, inspect, and update a Rudr application and its components

This guide covers how to install and configure a basic Rudr installation. For more details on Rudr installation, see the [Installation Guide](../setup/install.md)

## Prerequisites

The following prerequisites are required for a successful use of Rudr.

1. A copy of this repo (`git clone https://github.com/oam-dev/rudr.git`)
2. A Kubernetes cluster version 1.15 or greater
3. `kubectl` installed and pointed at the cluster
4. [Helm 3](https://v3.helm.sh/)

To find out which cluster Rudr would install to, you can run `kubectl config current-context` or `kubectl cluster-info`.

```console
$ kubectl config current-context
my-cluster
```

## Install Rudr and Dependencies

The fastest way to install Rudr is with Helm 3.

> make sure your Helm 3 has version newer than `v3.0.0-beta.3`, or you have to install the CRDs with `kubectl apply -f charts/rudr/crds`

```console
$ helm install rudr charts/rudr
NAME: rudr
LAST DEPLOYED: 2019-10-02 13:57:33.158655 -0600 MDT m=+5.183858344
NAMESPACE: default
STATUS: deployed
NOTES:
Rudr is a Kubernetes controller to manage Configuration CRDs.

It has been successfully installed.
```

This will give you a basic installation of Rudr. For the following examples, you should also install the NGINX ingress into Kubernetes:

```console
$ helm install nginx-ingress stable/nginx-ingress
NAME: nginx-ingress
LAST DEPLOYED: 2019-10-02 13:57:57.444655 -0600 MDT m=+2.129323603
NAMESPACE: default
STATUS: deployed
NOTES:
The nginx-ingress controller has been installed.
It may take a few minutes for the LoadBalancer IP to be available.
You can watch the status by running 'kubectl --namespace default get services -o wide -w nginx-ingress-controller'

...
```

This will give you a basic implementation of Kubernetes ingresses. See the [Installation Guide](../setup/install.md) for more about ingresses and other traits.

## Using Rudr

Once you have installed Rudr, you can start creating and deploying apps.

To start, install an example component:

```console
$ kubectl apply -f examples/helloworld-python-component.yaml
```

This component declares a simple web app written in Python. You can read the [Create Component from Scratch](../how-to/create_component_from_scratch.md) doc to know how we build it.

After that, you can list all available components using `kubectl`:

```console
$ kubectl get componentschematics
NAME              AGE
helloworld-python-v1   14s
```

You can look at an individual component:

```console
$ kubectl get componentschematic helloworld-python-v1 -o yaml
apiVersion: core.oam.dev/v1alpha1
kind: ComponentSchematic
metadata:
  creationTimestamp: "2019-10-08T13:02:23Z"
  generation: 1
  name: helloworld-python-v1
  namespace: default
  resourceVersion: "1989944"
  ...
spec:
  containers:
  - env:
    - fromParam: target
      name: TARGET
# ... more YAML
```

### Viewing Traits

Rudr provides a way to attach operational features at install time.
This allows application operations an opportunity to provide functionality like autoscaling,
caching, or ingress control at install time, without requiring the developer to change anything in the component.

You can also list the traits that are available on Rudr:

```console
$ kubectl get traits
NAME            AGE
autoscaler      19m
ingress         19m
manual-scaler   19m
```

And you can look at an individual trait in the same way that you investigate a component:

```console
$ kubectl get trait ingress -o yaml
apiVersion: core.oam.dev/v1alpha1
kind: Trait
metadata:
  creationTimestamp: "2019-10-02T19:57:37Z"
  generation: 1
  name: ingress
  namespace: default
  resourceVersion: "117813"
  selfLink: /apis/core.oam.dev/v1alpha1/namespaces/default/traits/ingress
  uid: 9f82c346-c8c6-4780-9949-3ecfd47879f9
spec:
  appliesTo:
  - core.oam.dev/v1alpha1.Server
  - core.oam.dev/v1alpha1.SingletonServer
  properties:
  - description: Host name for the ingress
    name: hostname
    required: true
    type: string
  - description: Port number on the service
    name: service_port
    required: true
    type: int
  - description: Path to expose. Default is '/'
    name: path
    required: false
    type: string
```

The above describes a trait that attaches an ingress to a component, handling the routing of traffic to that app.

## Installing an Application Configuration

When you are ready to try installing something, take a look at the `examples/first-app-config.yaml`, which shows a basic Application Configuration with a single trait applied:

```yaml
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
          value: '9999'
      traits:
        - name: ingress
          parameterValues:
            - name: hostname
              value: example.com
            - name: path
              value: /
            - name: service_port
              value: 9999
```

This is an example of an application composed of a singular component that has an ingress trait with an address of `example.com` and a service port of `9999`. 

To install this application configuration, use `kubectl`:

```console
$ kubectl apply -f examples/first-app-config.yaml
configuration.core.oam.dev/first-app created
```

You'll need to wait for a minute or two for it to fully deploy. Behind the scenes, Rudr is creating all the necessary objects.

Once it is fully deployed, you can see your configuration:

```console
$ kubectl get configurations
NAME        AGE
first-app   4m23s
$ kubectl get configuration first-app -o yaml
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  annotations:
     ...
  creationTimestamp: "2019-10-08T12:39:07Z"
  generation: 6
  name: first-app
  namespace: default
  resourceVersion: "2020150"
  selfLink: /apis/core.oam.dev/v1alpha1/namespaces/default/applicationconfigurations/first-app
  uid: 2ea9f384-993c-42b0-803a-43a1c273d291
spec:
  components:
  - instanceName: first-app-helloworld-python-v1
    name: helloworld-python-v1
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
status:
  components:
    helloworld-python-v1:
      deployment/first-app-helloworld-python-v1: running
      ingress/first-app-helloworld-python-v1-trait-ingress: Created
      service/first-app-helloworld-python-v1: created
  phase: synced
```

## Visit the web app

The way to visit the web app could be different with different platforms.
Assuming you are using [minikube](https://github.com/kubernetes/minikube),
you can view your web app from following steps:

1. Get the IP of minikube.
    ```shell script
    $ minikube ip
    192.168.99.101
    ``` 
2. Append hostname with IP to your hosts file, you can directly edit the file with `vim` or other tools.
   ```shell script
   echo "192.168.99.101 example.com" >> /etc/hosts
   ```
3. Then you can directly find what you want by `curl`
    ```shell script
    $ curl example.com
    Hello Rudr!
    ```

## Upgrade the Application Configuration file

Now we have successfully installed our web app and checked the result, the application worked well.
But someday, the operator may want to change something. For example:

1. the hostname: maybe because of there's conflict with other app, assume we change the hostname to `example2.com`.
2. env(target): this could represent some normal case of update, assume we change value of `target` to `World`.

### Change the Application Configuration file

So you could change `first-app-config.yaml` like below:

```yaml
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
-         value: Rudr
+         value: World
        - name: port
          value: '9999'
      traits:
        - name: ingress
          parameterValues:
            - name: hostname
-             value: example.com
+             value: example2.com
            - name: path
              value: /
            - name: service_port
              value: 9999
```

### Apply the changed file

Again we apply this yaml:

```console
$ kubectl apply -f examples/first-app-config.yaml
applicationconfiguration.core.oam.dev/first-app configured
```

### Check the updated app

Then check the applied yaml first:

```console
$ kubectl get configuration first-app -o yaml
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  annotations:
    ...
  creationTimestamp: "2019-10-08T12:39:07Z"
  generation: 9
  name: first-app
  namespace: default
  resourceVersion: "2022598"
  selfLink: /apis/core.oam.dev/v1alpha1/namespaces/default/applicationconfigurations/first-app
  uid: 2ea9f384-993c-42b0-803a-43a1c273d291
spec:
  components:
  - instanceName: first-app-helloworld-python-v1
    name: helloworld-python-v1
    parameterValues:
    - name: target
      value: World
    - name: port
      value: "9999"
    traits:
    - name: ingress
      parameterValues:
      - name: hostname
        value: example2.com
      - name: path
        value: /
      - name: service_port
        value: 9999
status:
  components:
    helloworld-python-v1:
      deployment/first-app-helloworld-python-v1: running
      ingress/first-app-helloworld-python-v1-trait-ingress: Created
      service/first-app-helloworld-python-v1: created
  phase: synced
``` 

You can see fields have been changed.

As we changed the hostname, we may set the hosts file again:

```shell script
echo "192.168.99.101 example2.com" >> /etc/hosts
```

Let's visit the web app again with the new hostname:

```console
$ curl example2.com
Hello World!
```

The response from the url indicates our change of environment has successfully went into effect.

## Upgrade with Component Changed

Assume several days have gone and the developer have developed a new version of the web app.

For example we change prefix of the response from `Hello` to `Goodbye`, and make a new component called `helloworld-python-v2`.
You can find more details about how we create it in [Upgrade Component](../how-to/create_component_from_scratch.md#Upgrade the component).

### Change and Apply the Application Configuration file

We need to change and apply the configuration file to make the component upgrade work.
 
```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ApplicationConfiguration
metadata:
  name: first-app
spec:
  components:
-   - name: helloworld-python-v1
+   - name: helloworld-python-v2
-     instanceName: first-app-helloworld-python-v1
+     instanceName: first-app-helloworld-python-v2
      parameterValues:
        - name: target
          value: World
        - name: port
          value: '9999'
      traits:
        - name: ingress
          parameterValues:
            - name: hostname
              value: example2.com
            - name: path
              value: /
            - name: service_port
              value: 9999
```

Apply it:
 
```shell script
$ kubectl apply -f examples/first-app-config.yaml
applicationconfiguration.core.oam.dev/first-app configured
```

### Check the upgrade result

You could check the applied yaml again by yourself. You should find the component name has been changed.

Let's visit the website directly:

```console
$ curl example2.com
Goodbye World!
```

Yeah, the updated web app worked very well!

Now we have successfully made our new component work.
This could be easier as the developer only need to care about component update while the operator only need to care about application configuration.

## Uninstalling Applications

You can delete your configurations easily with `kubectl`:

```console
$ kubectl delete configuration first-app
configuration.core.oam.dev "first-app" deleted
```

That will delete your application and all associated resources.

It will _not_ delete the traits and the components, they are happily waiting your use in the next Application Configuration.

```console
$ kubectl get traits,components
NAME                                AGE
trait.core.oam.dev/autoscaler      31m
trait.core.oam.dev/empty           31m
trait.core.oam.dev/ingress         31m
trait.core.oam.dev/manual-scaler   31m

NAME                                             AGE
component.core.oam.dev/alpine-replicable-task   19h
component.core.oam.dev/alpine-task              19h
component.core.oam.dev/hpa-example-replicated   19h
component.core.oam.dev/nginx-replicated         19h
component.core.oam.dev/nginx-singleton          19h
```

## Uninstall Rudr

If you want to clean up your test environment and uninstall Rudr, you could do the following:

 ```console
 $ helm delete rudr
 ```
 
 This will leave the CRDs and configurations intact.
 
 **NOTE: When you delete the CRDs, it will delete everything touching Open Application Model from configurations to secrets.**
 
 ```console
 kubectl delete crd -l app.kubernetes.io/part-of=core.oam.dev
 ```
 
 The above will delete the CRDs and clean up everything related with Open Application Model.

## Learn more

To learn more about Rudr, check out the [Using Rudr](../how-to/using_rudr.md) guide.
