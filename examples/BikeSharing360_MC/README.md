#BikeSharing360

During our Visual Studio 2017 Launch event this year, Scott Hanselman presented our Dockder Tooling experiences. 

This year, we built the technology stack for a fictional company named BikeSharing360, which allows users to rent bikes from one location to another.

BikeSharing360 is a fictitious example of a smart bike sharing system with 10,000 bikes distributed in 650 stations located throughout New York City and Seattle. Their vision is to provide a modern and personalized experience to riders and to run their business with intelligence.

In this demo scenario, we built several apps for both the enterprise and the consumer (bike riders). You can find all other BikeSharing360 repos in the following locations:

*[Mobile Apps](https://github.com/Microsoft/BikeSharing360_MobileApps)
*[Backend Services](https://github.com/Microsoft/BikeSharing360_BackendServices)
*[Websites](https://github.com/Microsoft/BikeSharing360_Websites)
*[Single Container Apps](https://github.com/Microsoft/BikeSharing360_SingleContainer)
*[Multi Container Apps](https://github.com/Microsoft/BikeSharing360_MultiContainer)
*[Cognitive Services Kiosk App](https://github.com/Microsoft/BikeSharing360_CognitiveServicesKioskApp)
*[Azure Bot App](https://github.com/Microsoft/BikeSharing360_BotApps)
# Multi Container Apps
Demo from Connect() 2016, where Donovan Brown opened an existing, "more complex" application than https://github.com/SteveLasker/Bikesharing360-Single to demonstrate seetting up Continuous Delivery with Visual Studio 2017 RC. 
The project was then deployed to Azure Container Services, through the Azure Container Registry.

# Building the project in a container
To validate the VSTS Build Steps will sucessfuly build the project, in a container, you can validate this locally:

How this works:

* when you call `docker-compose -f docker-compose.ci.build.yml up`, the image `microsoft/aspnetcore-build:1.0-1.1` is attempted to be instanced. 
* The first time, `microsoft/aspnetcore-build:1.0-1.1` isn't available compose up will build the image using  `.\build\Dockerfile` 
* The root of the solution is volume mapped in
* `dotnet restore`, `dotnet publish -c release` are executed

In the same folder as this readme.md file, call:
```
docker-compose -f docker-compose.ci.build.yml up
```

## directly on your dev machine

Once built, `cd .\bin\Release\publishoutput\`
From the published directory, `dotnet marketing.dll`
This will run the site at http://localhost:5000

## run in a container

Once built, `docker-compose up -d`
Find the dynamically assigned port: `docker ps`
```
 IMAGE                   PORTS
 bikesharing/marketing   0.0.0.0:32786->8080/tcp
```
http://localhost:32767

## How to sign up for Microsoft Azure

You need an Azure account to work with this demo code. You can:

- Open an Azure account for free [Azure subscription](https://azure.com). You get credits that can be used to try out paid Azure services. Even after the credits are used up, you can keep the account and use free Azure services and features, such as the Web Apps feature in Azure App Service.
- [Activate Visual Studio subscriber benefits](https://www.visualstudio.com/products/visual-studio-dev-essentials-vs). Your Visual Studio subscription gives you credits every month that you can use for paid Azure services.
- Not a Visual Studio subscriber? Get a $25 monthly Azure credit by joining [Visual Studio Dev Essentials](https://www.visualstudio.com/products/visual-studio-dev-essentials-vs).

## Blogs posts

Here's links to blog posts related to this project:

- Xamarin Blog: [Microsoft Connect(); 2016 Recap](https://blog.xamarin.com/microsoft-connect-2016-recap/)
- The Visual Studio Blog: [Announcing the new Visual Studio for Mac](https://blogs.msdn.microsoft.com/visualstudio/2016/11/16/visual-studio-for-mac/)
- The Visual Studio Blog: [Introducing Visual Studio Mobile Center (Preview)](https://blogs.msdn.microsoft.com/visualstudio/2016/11/16/visual-studio-mobile-center/)
- The Visual Studio Blog: [Visual Studio 2017 Release Candidate](https://blogs.msdn.microsoft.com/visualstudio/2016/11/16/visual-studio-2017-rc/)

## Clean and Rebuild
If you see build issues when pulling updates from the repo, try cleaning and rebuilding the solution.

## Copyright and license
* Code and documentation copyright 2016 Microsoft Corp. Code released under the [MIT license](https://opensource.org/licenses/MIT).

## Code of Conduct 
This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/). For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.
