use async_trait::async_trait;
use failure::Error;
use log::info;
use std::collections::BTreeMap;

mod server;
pub use crate::workload_type::server::{ReplicatedServer, SingletonServer};

mod task;
pub use crate::workload_type::task::{ReplicatedTask, SingletonTask};

mod worker;
pub use crate::workload_type::worker::{ReplicatedWorker, SingletonWorker};

mod workload_builder;
pub use crate::workload_type::workload_builder::WorkloadMetadata;

mod statefulset_builder;

pub mod extended_workload;

pub const OAM_API_VERSION: &str = "core.oam.dev/v1alpha1";

/// Server is a replicable server
pub const SERVER_NAME: &str = "core.oam.dev/v1alpha1.Server";
/// SingletonServer is a kind of Server that can't be replicated
pub const SINGLETON_SERVER_NAME: &str = "core.oam.dev/v1alpha1.SingletonServer";

/// SingletonTask is a task that cannot be replicated
pub const SINGLETON_TASK_NAME: &str = "core.oam.dev/v1alpha1.SingletonTask";
/// Task just means a replicated task which will replace ReplicatedTask
pub const TASK_NAME: &str = "core.oam.dev/v1alpha1.Task";

/// Singleton worker is a Worker that cannot be replicated
pub const SINGLETON_WORKER: &str = "core.oam.dev/v1alpha1.SingletonWorker";
/// Worker is daemon process that does not listen on the network
pub const WORKER_NAME: &str = "core.oam.dev/v1alpha1.Worker";

type InstigatorResult = Result<(), Error>;
type StatusResult = Result<BTreeMap<String, String>, Error>;
pub type ParamMap = BTreeMap<String, serde_json::Value>;
pub type ValidationResult = Result<(), failure::Error>;

/// KubeName describes anything that can produce its own Kubernetes name.
///
/// Most Kubernetes objects have their own name, and workload types, traits, and
/// other OAM objects are capable of autogenerating their own names in a
/// repeatable fashion. This trait describes the ability to repeatably create
/// a name.
///
/// KubeNames implementations should produce the same name for a given release. That
/// is, names are not random.
pub trait KubeName {
    fn kube_name(&self) -> String;
}

/// WorkloadType describes one of the available workload types.
///
/// An implementation of a workload type must be able to add, modify, and delete itself.
#[async_trait]
pub trait WorkloadType: Send + Sync {
    /// Add is responsible for installing the workload type into the cluster.
    async fn add(&self) -> InstigatorResult;
    /// Modify is responsible for upgrading an existing workload type.
    async fn modify(&self) -> InstigatorResult {
        Err(format_err!("Not implemented"))
    }
    /// Delete is responsible for deleting an existing workload type from the cluster.BTreeMap
    ///
    /// In the present implementation, most deletion logic relies upon Kubernetes' owner
    /// references, so there is no assurance that a deletion will be called for a workload.
    async fn delete(&self) -> InstigatorResult {
        info!("Workload deleted");
        Ok(())
    }
    /// Status returns the currently recorded status for the workload type
    async fn status(&self) -> StatusResult {
        Err(format_err!("Not implemented"))
    }
    /// Validate a worker's configuration before adding or modifying.
    async fn validate(&self) -> ValidationResult {
        Ok(())
    }
}

pub enum CoreWorkloadType {
    SingletonServerType(SingletonServer),
    ReplicatedServerType(ReplicatedServer),
    SingletonTaskType(SingletonTask),
    ReplicatedTaskType(ReplicatedTask),
    ReplicatedWorkerType(ReplicatedWorker),
    SingletonWorkerType(SingletonWorker),
}

#[async_trait]
impl WorkloadType for CoreWorkloadType {
    async fn add(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonServerType(sing) => sing.add().await,
            CoreWorkloadType::ReplicatedServerType(repl) => repl.add().await,
            CoreWorkloadType::SingletonTaskType(task) => task.add().await,
            CoreWorkloadType::ReplicatedTaskType(task) => task.add().await,
            CoreWorkloadType::ReplicatedWorkerType(task) => task.add().await,
            CoreWorkloadType::SingletonWorkerType(task) => task.add().await,
        }
    }
    async fn modify(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonServerType(sing) => sing.modify().await,
            CoreWorkloadType::ReplicatedServerType(repl) => repl.modify().await,
            CoreWorkloadType::SingletonTaskType(task) => task.modify().await,
            CoreWorkloadType::ReplicatedTaskType(task) => task.modify().await,
            CoreWorkloadType::ReplicatedWorkerType(task) => task.modify().await,
            CoreWorkloadType::SingletonWorkerType(task) => task.modify().await,
        }
    }
    async fn delete(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonServerType(sing) => sing.delete().await,
            CoreWorkloadType::ReplicatedServerType(repl) => repl.delete().await,
            CoreWorkloadType::SingletonTaskType(task) => task.delete().await,
            CoreWorkloadType::ReplicatedTaskType(task) => task.delete().await,
            CoreWorkloadType::ReplicatedWorkerType(task) => task.delete().await,
            CoreWorkloadType::SingletonWorkerType(task) => task.delete().await,
        }
    }
    async fn status(&self) -> StatusResult {
        match self {
            CoreWorkloadType::SingletonServerType(sing) => sing.status().await,
            CoreWorkloadType::ReplicatedServerType(repl) => repl.status().await,
            CoreWorkloadType::SingletonTaskType(task) => task.status().await,
            CoreWorkloadType::ReplicatedTaskType(task) => task.status().await,
            CoreWorkloadType::ReplicatedWorkerType(task) => task.status().await,
            CoreWorkloadType::SingletonWorkerType(task) => task.status().await,
        }
    }
    async fn validate(&self) -> ValidationResult {
        match self {
            CoreWorkloadType::SingletonServerType(sing) => sing.validate().await,
            CoreWorkloadType::ReplicatedServerType(repl) => repl.validate().await,
            CoreWorkloadType::SingletonTaskType(task) => task.validate().await,
            CoreWorkloadType::ReplicatedTaskType(task) => task.validate().await,
            CoreWorkloadType::ReplicatedWorkerType(task) => task.validate().await,
            CoreWorkloadType::SingletonWorkerType(task) => task.validate().await,
        }
    }
}

pub enum ExtendedWorkloadType {
    OpenFaaS(extended_workload::openfaas::OpenFaaS),
    Others(extended_workload::others::Others),
}

#[async_trait]
impl WorkloadType for ExtendedWorkloadType {
    async fn add(&self) -> InstigatorResult {
        match self {
            ExtendedWorkloadType::OpenFaaS(faas) => faas.add().await,
            ExtendedWorkloadType::Others(other) => other.add().await,
        }
    }
    async fn modify(&self) -> InstigatorResult {
        match self {
            ExtendedWorkloadType::OpenFaaS(faas) => faas.modify().await,
            ExtendedWorkloadType::Others(other) => other.modify().await,
        }
    }
    async fn delete(&self) -> InstigatorResult {
        match self {
            ExtendedWorkloadType::OpenFaaS(faas) => faas.delete().await,
            ExtendedWorkloadType::Others(other) => other.delete().await,
        }
    }
    async fn status(&self) -> StatusResult {
        match self {
            ExtendedWorkloadType::OpenFaaS(faas) => faas.status().await,
            ExtendedWorkloadType::Others(other) => other.status().await,
        }
    }
    async fn validate(&self) -> ValidationResult {
        match self {
            ExtendedWorkloadType::OpenFaaS(faas) => faas.validate().await,
            ExtendedWorkloadType::Others(other) => other.validate().await,
        }
    }
}
