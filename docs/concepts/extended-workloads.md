# Extended Workloads

The *workload type* (`workloadType`) is a field in the [component schematic](./component-schematic.md) used by the developer to direct the runtime to properly execute the given component.
[Extended workload types](https://github.com/oam-dev/spec/blob/master/3.component_model.md#extended-workload-types) are also available in rudr.

Rudr support two kinds of extended workload now:

1. **Inner extended workload**: this kind of workload was implemented in rudr project. So rudr will parse the workload setting and create the workload object.
2. **Fixed mod extended Workload**: this kind of workload wasn't implemented in rudr, but they have to put their spec in the workload setting.

## Inner extended workload

Now rudr has implemented OpenFaaS as an inner extended workload. You could read the [openfaas workload tutorial](../tutorial/deploy_openfaas_workload.md) to know how to play with it.
