FROM microsoft/aspnetcore:2.0-stretch AS base
WORKDIR /app
EXPOSE 80

FROM microsoft/aspnetcore-build:2.0-stretch AS build
WORKDIR /src
COPY ["Feedback.Api/Bikesharing.Feedback.Api.csproj", "Feedback.Api/"]
RUN dotnet restore "Feedback.Api/Bikesharing.Feedback.Api.csproj"
COPY . .
WORKDIR "/src/Feedback.Api"
RUN dotnet build "Bikesharing.Feedback.Api.csproj" -c Release -o /app/build

FROM build AS publish
RUN dotnet publish "Bikesharing.Feedback.Api.csproj" -c Release -o /app/publish

FROM base AS final
WORKDIR /app
COPY --from=publish /app/publish .
ENTRYPOINT ["dotnet", "Bikesharing.Feedback.Api.dll"]