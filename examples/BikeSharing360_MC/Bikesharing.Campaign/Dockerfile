FROM microsoft/aspnetcore:2.0-stretch AS base
WORKDIR /app
EXPOSE 80

FROM microsoft/aspnetcore-build:2.0-stretch AS build
WORKDIR /src
COPY ["Bikesharing.Campaign/Bikesharing.Campaign.csproj", "Bikesharing.Campaign/"]
COPY ["Core/Bikesharing.Core.csproj", "Core/"]
RUN dotnet restore "Bikesharing.Campaign/Bikesharing.Campaign.csproj"
COPY . .
WORKDIR "/src/Bikesharing.Campaign"
RUN dotnet build "Bikesharing.Campaign.csproj" -c Release -o /app/build

FROM build AS publish
RUN dotnet publish "Bikesharing.Campaign.csproj" -c Release -o /app/publish

FROM base AS final
WORKDIR /app
COPY --from=publish /app/publish .
ENTRYPOINT ["dotnet", "Bikesharing.Campaign.dll"]