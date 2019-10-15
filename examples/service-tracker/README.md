# Service Tracker Example

## Contents 

This folder contains the source code for the Service Tracker sample. Microservices for each of the services (Flight Tracking, Earthquake tracking, Weather) are provided in the `app` folder. The YAML files in this folder contain the OAM components and app configuration. 

Full credit for this sample goes to Brian Redmond. Original source can be found here: https://github.com/chzbrgr71/service-tracker. 

## Deploy

```bash
kubectl create -f tracker-components.yaml
kubectl create -f tracker-app-config.yaml

kubectl delete -f tracker-app-config.yaml
kubectl delete -f tracker-components.yaml
```