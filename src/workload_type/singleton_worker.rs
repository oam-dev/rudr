use k8s_openapi::api::batch::v1 as batchapi;
use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;

use crate::schematic::component::Component;
use crate::workload_type::{InstigatorResult, KubeName, ParamMap, WorkloadType, workload_builder::JobBuilder};

use std::collections::BTreeMap;


