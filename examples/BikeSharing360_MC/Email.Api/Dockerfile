FROM microsoft/aspnetcore:2.0-stretch AS base
WORKDIR /app
EXPOSE 80

FROM microsoft/aspnetcore-build:2.0-stretch AS build
WORKDIR /src
COPY ["Email.Api/Bikesharing.Email.Api.csproj", "Email.Api/"]
RUN dotnet restore "Email.Api/Bikesharing.Email.Api.csproj"
COPY . .
WORKDIR "/src/Email.Api"
RUN dotnet build "Bikesharing.Email.Api.csproj" -c Release -o /app/build

FROM build AS publish
RUN dotnet publish "Bikesharing.Email.Api.csproj" -c Release -o /app/publish

FROM base AS final
WORKDIR /app
COPY --from=publish /app/publish .
ENTRYPOINT ["dotnet", "Bikesharing.Email.Api.dll"]