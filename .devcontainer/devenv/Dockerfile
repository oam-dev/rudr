#-------------------------------------------------------------------------------------------------------------
# Copyright (c) Microsoft Corporation. All rights reserved.
# Licensed under the MIT License. See https://go.microsoft.com/fwlink/?linkid=2090316 for license information.
#-------------------------------------------------------------------------------------------------------------

FROM ubuntu:18.04

# Avoid warnings by switching to noninteractive
ENV DEBIAN_FRONTEND=noninteractive

# Install prereqs
RUN apt-get update && apt-get install -y apt-transport-https curl wget gnupg2 git procps

# Install k8s
RUN curl -s https://packages.cloud.google.com/apt/doc/apt-key.gpg | apt-key add - \
    && echo "deb https://apt.kubernetes.io/ kubernetes-xenial main" | tee -a /etc/apt/sources.list.d/kubernetes.list \
    && apt-get update \
    && apt-get install -y kubectl

# Install Helm 3
RUN wget https://get.helm.sh/helm-v3.0.0-beta.5-linux-amd64.tar.gz \
    && tar -zxvf helm-v3.0.0-beta.5-linux-amd64.tar.gz \
    && mv linux-amd64/helm /usr/local/bin/helm

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  -s -- -y \
    && apt install build-essential libssl-dev pkg-config -y \
    && echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc

# Switch back to dialog for any ad-hoc use of apt-get
ENV DEBIAN_FRONTEND=dialog
