# How to Debug?

This document outlines troubleshooting guides to help you debug things when deployments via Rudr are not successful.

## Check Rudr pod is running

Make sure rudr is running successfully.
 
```
$ kubectl get pods -l app.kubernetes.io/name=rudr
NAME                    READY   STATUS    RESTARTS   AGE
rudr-6b9b9c57cd-zxgfr   1/1     Running   0          30s
```

## Check Rudr logs

When you deploy an Application Configuration, you could find information about it from logs.

```
$ kubectl logs -f rudr-6b9b9c57cd-zxgfr 
```

Currently, all workload implemented in Rudr will create pods.
So if your ApplicationConfiguration didn't create any pod, there must be some error information in the logs.
Check the logs and file an issue with the log information. 


## Check if the Component exists

```shell script
$ kubectl get comp
NAME                         AGE
alpine-singleton-task-v1     3s
alpine-singleton-worker-v1   3s
alpine-task-v1               3s
alpine-worker-v1             3s
hpa-example-replicated-v1    3s
nginx-replicated-v1          3s
nginx-singleton-v1           3s
```

Don't forget to deploy component before using them.

## Check parameters

Check the parameters and properties of ApplicationConfiguration are consistent with trait and component.

## Pods is created but unexpected behavior

For example incorrect metadata, trait unavailable. Check the logs of Rudr pod, if you couldn't solve it, feel free to create an issue.
