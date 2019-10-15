# Overview of Samples

This directory contains samples and examples used throughout the Rudr documentation. This document outlines what each sample attempts to show and can be used as a reference for. 

| File Name        | Description
|-|-|
| service-tracker.yaml| This folder contains manifests and source for the service tracker sample. It is a multi-component application that uses environment variables, ingress trait and the manual scaler trait.|
| voting.yaml | This folder contains manifests for the canonical voting sample. It is a multi-component application that uses environment variables and the ingress trait. |
| autoscaler.yaml| This is an example of how to use the autoscaler trait. |
| components.yaml| General components that are used in quickstarts, how-to's and instantiated by the app configurations in this folder. |
| env-vars.yaml| This is an example of how to specify environment variables for your containers. |
| first-app-config.yaml| This is the configuration applied to the `nginx-component` as part of the quickstarts. |
| helloworld-python-component.yaml| This component is the complete version of the **Create Component from Scratch** [guide](../docs/how-to/create_component_from_scratch.md) |
| image-pull-secret.yaml| This is an example of how to specify secrets for private registries|
| manual-scaler.yaml| This is an example of to manually specify replica counts for components. |
| multi-component.yaml| This is an example of how to specify multiple components in one manifest. |
| multi-server.yaml| This is an example of an app configuration instantiating multiple components. |
| nginx-component.yaml| Basic NGINX component used for quickstarts. |
| replicable-task.yaml| This is an example of an app config instantiating replicable tasks.|
| task.yaml| This is an example of an app config instantating a simple task to completion. |
| volumes.yaml| This is an example of how to specify volumes required in the component schematics. |
| worker.yaml| This is an example of how to instantiate a worker component. |

