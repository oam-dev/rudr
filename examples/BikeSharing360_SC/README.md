# BikeSharing360 with Rudr

[BikeSharing360 for single containers](https://github.com/microsoft/BikeSharing360_SingleContainer) adapted for Rudr.

BikeSharing360 is a fictitious example of a smart bike sharing system with 10,000 bikes distributed in 650 stations located throughout New York City and Seattle. Their vision is to provide a modern and personalized experience to riders and to run their business with intelligence.

BikeSharing 360 for multiple containers can be found [here](https://github.com/Microsoft/BikeSharing360_MultiContainer).

# Building the project

Make sure that the ingress controller has been installed (instructions [here](../../docs/setup/install.md)).

Add the components:

```
> kubectl apply -f Manifest/bikeshareing-sc-component.yaml

```

Validate that the components were created successfully:

```
> kubectl get comp     

NAME                          AGE
bikesharing-sc-component-v1   24s
     
```

Apply the configuration:

```
> kubectl apply -f Manifest/bikeshare-sc-config.yaml
```

Validate that the configuration was applied successfully:

```
> kubectl get configurations

NAME              AGE
bikesharing-app   25s
```

Wait for the ingress to be created:

```
> kubectl get ingress                                                             
NAME                                           HOSTS             ADDRESS   PORTS   AGE
bikesharing-app-v1-trait-ingress               bikesharing.com             80      1s
first-app-helloworld-python-v1-trait-ingress   rudr.hello.com              80      47h
```

Navigating to bikesharing.com after mapping it to the correct IP address (found by `kubectl get services`) should take you to the home page.

![home-page](bikesharing-sc-page.png "Home Page")

## Blogs posts

Here's links relevant to this project:

- OAM project: [official website](https://oam.dev/)
- Open Source Blog: [OAM release announcement](https://cloudblogs.microsoft.com/opensource/2019/10/16/announcing-open-application-model/)

## Copyright and license
* Code and documentation copyright 2016 Microsoft Corp. Code released under the [MIT license](https://opensource.org/licenses/MIT).

## Code of Conduct 
This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/). For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.
