# Governance

## LGTM Policy

Every code change committed to Scylla must be reviewed by at least one project maintainer who did not author the PR. When a PR has been marked `Approved` by a project maintainer and passes all mandatory gates (including the contributor license agreement verification), the PR can be merged by any project maintainer.

## Project Maintainers
[Project maintainers](CODEOWNERS) are responsible for activities around maintaining and updating Scylla. Final decisions on the project reside with the project maintainers.

Maintainers MUST remain active. If they are unresponsive for >3 months, they will be automatically removed unless a [super-majority](https://en.wikipedia.org/wiki/Supermajority#Two-thirds_vote) of the other project maintainers agrees to extend the period to be greater than 3 months.

New maintainers can be added to the project by a [super-majority](https://en.wikipedia.org/wiki/Supermajority#Two-thirds_vote) vote of the existing maintainers.

A maintainer may step down by submitting an [issue](https://github.com/microsoft/scylla/issues/new) stating their intent.

## Triaging and Milestones 

### Milestones
Currently, the Scylla project is tracking two milestones. 

| Milestone        | Description  |
|---------------------|---|
| 1.0.0-a1 | Milestone tracking items for the first alpha release of Scylla. Currently, this is slated to be released with the first release of the [specification](https://github.com/microsoft/hydra-spec). |
| 1.0.0-a2 | Milestone tracking items for the second alpha release of Scylla.  |

### Triaging 

Triaging of items into milestones will occur during the bi-weekly community call. During this call, issues might be brought into milestones, removed from milestones or moved between milestones. 

Issues are assigned ONE status tag, one (or more) type tags and ONE milestone. The tags for issues are described in the [labels](https://github.com/microsoft/scylla/labels). 

## Code of Conduct
This project has adopted the [Microsoft Open Source Code of conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.
