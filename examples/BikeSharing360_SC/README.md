#BikeSharing360

During our Visual Studio 2017 Launch event this year, Scott Hanselman presented our Dockder Tooling experiences. 

This year, we built the technology stack for a fictional company named BikeSharing360, which allows users to rent bikes from one location to another.

BikeSharing360 is a fictitious example of a smart bike sharing system with 10,000 bikes distributed in 650 stations located throughout New York City and Seattle. Their vision is to provide a modern and personalized experience to riders and to run their business with intelligence.

In this demo scenario, we built several apps for both the enterprise and the consumer (bike riders). You can find all other BikeSharing360 repos in the following locations:

* [Mobile Apps](https://github.com/Microsoft/BikeSharing360_MobileApps)
* [Backend Services](https://github.com/Microsoft/BikeSharing360_BackendServices)
* [Websites](https://github.com/Microsoft/BikeSharing360_Websites)
* [Single Container Apps](https://github.com/Microsoft/BikeSharing360_SingleContainer)
* [Multi Container Apps](https://github.com/Microsoft/BikeSharing360_MultiContainer)
* [Cognitive Services Kiosk App](https://github.com/Microsoft/BikeSharing360_CognitiveServicesKioskApp)
* [Azure Bot App](https://github.com/Microsoft/BikeSharing360_BotApps)

# Bikesharing360 Single Container App
In this repo you will find the Single Container demo where Scott opened an existing project, added docker support and published it to Azure App service, running Linux Docker containers.

## Demo Prerequisites
The Single Container demo encompasses taking an existing ASP.NET Core Web App and deploying it to Azure App Services for Linux, using container deployments. To run this demo you'll need:

* An Azure Subscription (see below)
* [Visual Studio 2017 RC](https://www.visualstudio.com/vs/visual-studio-2017-rc/) Choose the **".NET Core & Docker (Preview)"** Workload
* [Docker for Windows](https://www.docker.com/products/docker#/windows) Used to run Docker Containers Locally, including the Linux containers used in this demo.

The creation and publishing to an Azure App Service will be done as part of the demo.   

## How to sign up for Microsoft Azure

To run this demo, you'll need an Azure account. You can:

- Open an Azure account for free [Azure subscription](https://azure.com). You get credits that can be used to try out paid Azure services. Even after the credits are used up, you can keep the account and use free Azure services and features, such as the Web Apps feature in Azure App Service.
- [Activate Visual Studio subscriber benefits](https://www.visualstudio.com/products/visual-studio-dev-essentials-vs). Your Visual Studio subscription gives you credits every month that you can use for paid Azure services.
- Not a Visual Studio subscriber? Get a $25 monthly Azure credit by joining [Visual Studio Dev Essentials](https://www.visualstudio.com/products/visual-studio-dev-essentials-vs).

## Demo Steps

### Open and run the project within a container, making live changes
1. Clone the before docker branch and open this repo locally using Visual Studio 2017
2. Right Click the project and choose **Add** -> **Docker Support**
3. F5 - or click the |> Start Debugging, with Docker as the target
4. Open Views\Home\Index.cshtml
5. Add some text, such as "Come see what we've got" to the end of "...throughout New York City and Seattle." 
6. Save Index.cshtml
7. Refresh the page in the browser
8. Voila, you've now run a .NET Core app in a Linux container and made a change without having to rebuild or re-run the container image.

### Publish to Kubernetes using Scylla

1.	Make sure that the scyall pre-requistes and an ingress controller like nginx installed if you want test the app     outside the cluster.
2.  Run the following command under the Manifest folder to create the components, kubectl create -f bikesharing_components.yml
3.  Now, run the following command under the Manifest folder to create the operational configuration, kubectl create -f bikesharing_components.yml
4.  Run the following command to list out all of the objects created in kubernets. kubectl get componentschematic,OperationalConfiguration,pods,svc,ingress 

    An, e.g output shows below.

    NAME                                     AGE
    componentschematic.core.hydra.io/bs-ui   16m

    NAME                                           AGE
    operationalconfiguration.core.hydra.io/bs-ui   16m

    NAME                                              READY   STATUS    RESTARTS   AGE
    pod/bs-ui-c6774c996-wvc8n                         1/1     Running   0          16m
    pod/nginx-scylla-nginx-ingress-6889cb7886-z559g   1/1     Running   0          2d4h
    pod/scylla-9f7549bc-w6tns                         1/1     Running   0          4d5h

    NAME                                 TYPE           CLUSTER-IP     EXTERNAL-IP     PORT(S)                      AGE
    service/bs-ui                        ClusterIP      10.0.204.135   <none>          80/TCP                       16m
    service/kubernetes                   ClusterIP      10.0.0.1       <none>          443/TCP                      4d23h
    service/nginx-scylla-nginx-ingress   LoadBalancer   10.0.220.196   52.177.130.21   80:32303/TCP,443:30262/TCP   2d4h

    NAME                                     HOSTS               ADDRESS         PORTS   AGE
    ingress.extensions/bs-ui-trait-ingress   bikes.example.com   52.177.130.21   80      16m

Now, you should be able to access your application outside the cluster. You may need to add local host file entry to point to the external IP if you are testing from your local machine if the host is not registered in a DNS.

## Copyright and license
* Code and documentation copyright 2016 Microsoft Corp. Code released under the [MIT license](https://opensource.org/licenses/MIT).

## Code of Conduct 
This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/). For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

