# Voting Example

This is the Docker voting application exhibited as a Hydra application.

## Installation

The first step is to install all of the component descriptions:

```console
$ kubectl create -f components.yaml
$ kubectl get components
NAME              AGE
postgres          13s
redis             12s
voting-admin      13s
voting-frontend   12s
voting-worker     12s
```

Next, create a configuration:

```console
$ kubectl create -f configuration.yaml
```

In a few minutes, you should have the entire voting application running.