# Extended Workloads

The *workload type* (`workloadType`) is a field in the [component schematic](./component-schematic.md) used by the developer to direct the runtime to properly execute the given component.
[Extended workload types](https://github.com/oam-dev/spec/blob/master/3.component_model.md#extended-workload-types) are also available in rudr.

Rudr support two kinds of extended workload now:

1. **Intrusive extended workload**: this kind of workload must be implemented in rudr project. So the workload rely on rudr to parse the workload setting and create the workload object.
2. **Non-Intrusive Workload**: this kind of workload don't need to be implemented in rudr. Currently, they have to put their whole spec in the workload setting. We are designing a better way to organize the schema.

## Intrusive extended workload

Now rudr has implemented OpenFaaS as an intrusive extended workload. You could read the [openfaas workload tutorial](../tutorial/deploy_openfaas_workload.md) to know how to use it.


## Non-Intrusive Workload

Non-Intrusive Workload rely on how we write the workloadSetting. You could read the [prometheus workload tutorial](../tutorial/deploy_prometheus_workload.md) to know how to use it.