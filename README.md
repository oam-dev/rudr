# Scylla: A Kubernetes Hydra Implementation in Rust

This project implements the [Hydra specification](https://github.com/microsoft/hydra-spec) for Kubernetes.

**This is unstable, experimental, and subject to massively breaking changes. It may reflect the spec, or even features we are vetting before inclusion into the spec.**

## Building

To build:

- Make sure you have Rust 2018 Edition or newer
- Clone this repository
- Go into the main directory: `cd scylla`
- Install the CRDs in `example/crds.yaml`: `kubectl create -f examples/crds.yaml`
- Run `cargo build`
- To run the server: `cargo run`

At this point, you will be running a local controller attached to the cluster to which your kubeconfig is pointing.

To get started, create some _components_. Components are not instantiated. They are descriptions of what things can run in your cluster.

```console
$ kubectl create -f k8s/nginx-component.yaml
$ kubectl get components
NAME              AGE
nginx-component   17s
```

Next, create a new application that uses the component. In Hydra, as in 12-factor, an app is code (component) plus config. So you need to write a configuration. An example is provided in the `k8s/` directory:

```console
$ kubectl create -f k8s/first-app-config.yaml
```

Now you may wish to explore your cluster to see what was created:

```console
$ kubectl get configuration,pod,svc,ingress
NAME        AGE
first-app   13m

NAME        READY     STATUS    RESTARTS   AGE
first-app   1/1       Running   0          13m

NAME         TYPE        CLUSTER-IP     EXTERNAL-IP   PORT(S)   AGE
first-app    ClusterIP   10.0.213.123   <none>        80/TCP    13m
```

To delete this, just do a `kubectl delete configuration first-app` and it will cascade and delete all of the pieces.

## TODO

This is the _brief_ version of high level TODOs. There are plenty more things that actually need to be done.

- [ ] Handle parameters!!!
- [ ] Switch to logger (death to `println!`)
- [ ] Fix namespacing as soon as scopes make it into the spec
- [ ] clone() is a crutch (90% of the time)
- [ ] Refactor main()
- [ ] How do we make the trait system more flexible?
- [ ] Under what conditions is a deployment error a failure, a rollback, or a "keep going"?
- [ ] Can we use ownership references to cascade deletes without the controller?

## License

This project is available under the terms of the MIT license. See [LICENSE.txt](LICENSE.txt).

# Contributing

This project welcomes contributions and suggestions.  Most contributions require you to agree to a Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us the rights to use your contribution. For details, visit https://cla.microsoft.com.

When you submit a pull request, a CLA-bot will automatically determine whether you need to provide a CLA and decorate the PR appropriately (e.g., label, comment). Simply follow the instructions provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/). For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

## About the Name

Scylla is one of the monsters in Homer's Odyssey. Odysseus must steer his ship between Scylla and Charybdis. Scylla is sometimes portrayed as a hydra.