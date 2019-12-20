FROM microsoft/aspnetcore:2.0-stretch AS base
WORKDIR /app
EXPOSE 80

FROM microsoft/aspnetcore-build:2.0-stretch AS build
WORKDIR /src
COPY ["Profile.Api/Bikesharing.Profile.Api.csproj", "Profile.Api/"]
RUN dotnet restore "Profile.Api/Bikesharing.Profile.Api.csproj"
COPY . .
WORKDIR "/src/Profile.Api"
RUN dotnet build "Bikesharing.Profile.Api.csproj" -c Release -o /app/build

FROM build AS publish
RUN dotnet publish "Bikesharing.Profile.Api.csproj" -c Release -o /app/publish

FROM base AS final
WORKDIR /app
COPY --from=publish /app/publish .
ENTRYPOINT ["dotnet", "Bikesharing.Profile.Api.dll"]