# Create Component from Scratch

In this tutorial, we will build a simple web app component written in Python that you can use for testing.
It reads in an env variable TARGET and prints “Hello \${TARGET}!“.
If TARGET is not specified, it will use “World” as the TARGET.

## Prerequisites

* Follow the instructions in the [installation](../setup/install.md) document to get Rudr installed on your Kubernetes cluster.
* [Docker](https://www.docker.com/) installed and running on your local machine, and a [Docker Hub](https://hub.docker.com) account configured (we’ll use it for a container registry).

## Steps to build image

The following instructions will lead you to build an image from source, you can get all the files mentioned here in the [app](./app) folder.

1. Create a new directory and cd into it:
    ```shell script
    mkdir app
    cd app
    ```
2. Create a file named `app.py` and copy the code from [`app/app.py`](./app/app.py)
3. Create a file named `Dockerfile` and copy the code from [`app/Dockerfile`](./app/Dockerfile), See [official Python docker image](https://hub.docker.com/_/python/) for more details.
4. Use Docker to build the sample code into a container. To build and push with Docker Hub, run these commands replacing `oamdev` with your Docker Hub username:
    ```shell script
   # Build the container on your local machine
   docker build -t oamdev/helloworld-python:v1 .

   # Push the container to docker registry
   docker push oamdev/helloworld-python:v1
    ```
   
## Create Component Schematics File

Now we have a docker image named `oamdev/helloworld-python:v1`, so we can use this image to create a component schematics file.

1. Choose the workloadType: the `helloworld-python` is very typical web application, it is stateless, always running as a service, can be replicated. So we use `core.oam.dev/v1alpha1.Server` without doubt.
2. Fill the container spec and make ENV configurable: obviously we have two major environment variables in the image, one is TARGET and the other is PORT.
3. Make parameters so we could let Application Configuration configure these environments.

After these three concerns, we could figure out this basic component schematic yaml like below: 

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ComponentSchematic
metadata:
  name: helloworld-python-v1
spec:
  name: helloworld-python
  workloadType: core.oam.dev/v1alpha1.Server
  containers:
    - name: foo
      image: oamdev/helloworld-python:v1
      env:
        - name: TARGET
          fromParam: target
        - name: PORT
          fromParam: port
      ports:
        - type: tcp
          containerPort: 9999
          name: http
  parameters:
    - name: target
      type: string
      default: World
    - name: port
      type: string
      default: '9999'
```

Let's name it `helloworld-python-component.yaml` and put it into [the `examples` folder](../../examples/helloworld-python-component.yaml).

Finally we could apply this yaml to the platform and let our operators to deploy it.

```shell script
$ kubectl apply -f examples/helloworld-python-component.yaml
componentschematic.core.oam.dev/helloworld-python-v1 created
```

You can check if your component schematic is OK with:

```shell script
$ kubectl get comp
NAME                   AGE
helloworld-python-v1   15s
```

Yeah, we have successfully built a component from source now.

## Upgrade the component

We assume a component is immutable. If we want to upgrade a component,
the easiest way is to modify the component and change the name suffix with a new version.

### Change the code 

For example, we change the code from `Hello` to `Goodbye`.

```shell script
import os

from flask import Flask

app = Flask(__name__)

@app.route('/')
def hello_world():
    target = os.environ.get('TARGET', 'World')
-   return 'Hello {}!\n'.format(target)
+   return 'Goodbye {}!\n'.format(target)

if __name__ == "__main__":
    app.run(debug=True, host='0.0.0.0', port=int(os.environ.get('PORT', "8080")))
```

Build and create image with a new tag.

```shell script
docker build -t oamdev/helloworld-python:v2 .
docker push oamdev/helloworld-python:v2
``` 

### Change the component

Change the component with a new name.

```yaml
apiVersion: core.oam.dev/v1alpha1
kind: ComponentSchematic
metadata:
- name: helloworld-python-v1
+ name: helloworld-python-v2
spec:
  name: helloworld-python
  workloadType: core.oam.dev/v1alpha1.Server
  containers:
    - name: foo
-     image: oamdev/helloworld-python:v1
+     image: oamdev/helloworld-python:v2

      env:
        - name: TARGET
          fromParam: target
        - name: PORT
          fromParam: port
      ports:
        - type: tcp
          containerPort: 9999
          name: http
  parameters:
    - name: target
      type: string
      default: World
    - name: port
      type: string
      default: '9999'
```

Apply the changed component:

```console
$ kubectl apply -f examples/helloworld-python-component.yaml
componentschematic.core.oam.dev/helloworld-python-v2 created
```

### Check the result

Now we have two components:

```console
$ kubectl get comp
NAME                   AGE
helloworld-python-v1   1h
helloworld-python-v2   27s
```

They could be used by operator in application configuration.  
