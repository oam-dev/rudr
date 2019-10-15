# Example Charts of OAM Apps

This directory contains example Helm charts that install Open Application Model apps.

Helm is a useful tool for parameterizing AppConfig files. There are various strategies for installing ComponentSchematics:

- They may be bundled into the same chart that manages them, and treated like standard resources
    - When this chart is upgraded or deleted, components will be updated or deleted
    - This can be bad if multiple apps share the same components
    - This can be good if your app configs and components are closely related
- They may be bundled into the same chart that references them, but managed with hooks
    - You can configure hooks to not delete components
    - This solves some of the problems above
- They may be kept in separate Helm charts
    - This model is best if you want to have lots of components that can be shared among different app configs


## Installing OAM Apps

These examples are built for Helm 3. They can be installed with the following command:

```console
$ helm install my-hello hello-rudr
```

They can be uninstalled with `helm delete my-hello`.
