# Contributing

This project welcomes contributions and suggestions.  

All contributions require you to agree to a Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us the rights to use your contribution. For details, visit https://cla.microsoft.com.

When you submit a pull request, a CLA-bot will automatically determine whether you need to provide a CLA and decorate the PR appropriately (e.g., label, comment). Simply follow the instructions provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/). For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

## Building from Source

This section goes over how to build the source code for Rudr. 

### Prerequisites 

- [Rust 2018 Edition or newer](https://www.rust-lang.org/tools/install)
- Install kubectl and Helm 3. Instructions for both are in the [set up doc](./docs/setup/install.md)
- Access to a Kubernetes cluster. Instructions for [minikube](https://kubernetes.io/docs/tasks/tools/install-minikube/) can be found here 

To build:

- Clone this repository
- Go into the main directory: `cd scylla`
- Install the CRDs
```bash
kubectl apply -f charts/scylla/crds/appconfigs.yaml
kubectl apply -f charts/scylla/crds/componentinstances.yaml
kubectl apply -f charts/scylla/crds/componentschematics.yaml
kubectl apply -f charts/scylla/crds/scopes.yaml
kubectl apply -f charts/scylla/crds/traits.yaml
```
- Run `cargo build`
- To run the server: `make run`, this will run Rudr controller locally, and use the cluster by your `~/.kube/config`.

At this point, you will be running a local controller attached to the cluster to which your `$KUBECONFIG` is pointing.

To get started, define some _components_. Components are not instantiated. They are descriptions of what things can run in your cluster.

```console
$ kubectl apply -f examples/components.yaml
component.core.hydra.io "nginx-replicated" created
component.core.hydra.io "nginx-singleton" created
$ kubectl get components
NAME               AGE
nginx-replicated   17s
nginx-singleton    17s
```

Next, create a new application that uses the component. In Open Application Model, which follows the 12-factor model, the application is composed of code (component) and a config. So you need to write a configuration. Examples are provided in the `examples/` directory:

```console
$ kubectl apply -f examples/first-app-config.yaml
```

Now you may wish to explore your cluster to see what was created:

```console
$ kubectl get configuration,pod,svc,ingress
NAME        AGE
first-app   28s

NAME                        READY     STATUS    RESTARTS   AGE
first-app-nginx-singleton   1/1       Running   0          19s

NAME                                TYPE        CLUSTER-IP    EXTERNAL-IP   PORT(S)   AGE
first-app-nginx-singleton           ClusterIP   10.0.78.193   <none>        80/TCP    19s
kubernetes                          ClusterIP   10.0.0.1      <none>        443/TCP   95d

NAME                                      HOSTS         ADDRESS   PORTS     AGE
first-app-nginx-singleton-trait-ingress   example.com             80        19s
```

To delete this, run `kubectl delete configuration first-app` and it will cascade and delete all of the pieces.

## Contributing via pull requests

Like any good open source project, we use Pull Requests (PRs) to track code changes. Please familiarize yourself with the **Status** labels on the [labels](https://github.com/microsoft/scylla/labels) page. 

1. Fork the repo, modify to address the issue.
2. Link the PR to the issue. 
3. Submit a pull request.

### PR Lifecycle

1. PR creation
    - We more than welcome PRs that are currently in progress. They are a great way to keep track of
    important work that is in-flight, but useful for others to see. If a PR is a work in progress,
    it **should** be prefaced with "WIP: [title]" and add `Status: In Progress` will be added as a label. You should add the `Status: Review Needed` **label** once the PR is ready for review and remove "WIP" from the title.
    - It is preferred, but not required, to have a PR tied to a specific issue. There can be
    circumstances where if it is a quick fix then an issue might be overkill. The details provided
    in the PR description would suffice in this case.
2. Triage
    - The maintainer in charge of triaging will apply the proper labels for the issue. This should
    include at least a status label and a milestone.
3. Assigning reviews
    - All PRs require 1 review approval from a maintainer before being merged. 
4. Reviewing/Discussion
    - All reviews will be completed using Github review tool.
    - A "Comment" review should be used when there are questions about Rudr. This type of review does not count as approval.
    - A "Changes Requested" review indicates that changes need to be made before they will be
    merged.
    - Reviewers should update labels as needed (such as `Status: Needs rebase`).
    - When a review is approved, the reviewer should add `LGTM` as a comment. 
    - Final approval is required by a designated owner (see `.github/CODEOWNERS` file). Merging is blocked without this final approval. Approvers will factor reviews from all other reviewers into their approval process.
5. PR owner should try to be responsive to comments by answering questions or changing text. Once all comments have been addressed,
   the PR is ready to be merged. When it gets merged, the `Status: Completed` will be added signifying that it is in the next release candidate. 
6. Merge or close
    - A PR should stay open until a Final Approver (see above) has marked the PR approved
    - PRs can be closed by the author without merging
    - PRs may be closed by a Final Approver if the decision is made that the PR is not going to be merged 

## Contributing via Issues

There are more ways to contribute to open source projects than pull requests. We implore users to open issues with any suggestions or problems discovered. Issues are used as the primary method for tracking work in the milestones on the project.

### Issue Types

To learn about issue types, please read the [labels](https://github.com/microsoft/scylla/labels) page. 

### Issue Lifecycle

The issue lifecycle is mainly driven by the core maintainers, but is good information for those
contributing to Rudr. All issue types follow the same general lifecycle.

1. Issue creation. 
2. Triage
    - The maintainer in charge of triaging will apply the proper labels for the issue. This
    includes labels for type, projects/milestones and metadata.
    - (If needed) Clean up the title to succinctly and clearly state the issue. Also ensure
    that proposals are prefaced with "Proposal".
    - We attempt to do this process at least once per work day.
3. Discussion
    - Enhancement, bug and document issues should be connected to the PR that resolves it.
    - Whoever is working on an issue should claim the issue in the comments.
    - Issues should stay open until a maintainer closes it or the owner of the issue decides to close it. 
4. Issue closure.

## Milestones and Triaging 

### Milestones

To get an overview of the milestones that are being tracked for Rudr please visit the [Milestones](https://github.com/microsoft/hydra-spec/milestones) page. 

### Triaging 

Each day, someone from a Open Application Model related team should act as the triager. This person will be in charge triaging new PRs and issues throughout the day. Anyone can volunteer as the triager by posting on the Slack channel or volunteering in advance during our community calls. If no one has volunteered by 10:00 AM PST, someone from our team will triage. 

Broader discussion of any issues can be raised during the bi-weekly community call. Issues might be brought into milestones, removed from milestones or moved between milestones during the call.