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
pub trait WorkloadType {
    /// Add is responsible for installing the workload type into the cluster.
    fn add(&self) -> InstigatorResult;
    /// Modify is responsible for upgrading an existing workload type.
    fn modify(&self) -> InstigatorResult {
        Err(format_err!("Not implemented"))
    }
    /// Delete is responsible for deleting an existing workload type from the cluster.BTreeMap
    ///
    /// In the present implementation, most deletion logic relies upon Kubernetes' owner
    /// references, so there is no assurance that a deletion will be called for a workload.
    fn delete(&self) -> InstigatorResult {
        info!("Workload deleted");
        Ok(())
    }
    /// Status returns the currently recorded status for the workload type
    fn status(&self) -> StatusResult {
        Err(format_err!("Not implemented"))
    }
    /// Validate a worker's configuration before adding or modifying.
    fn validate(&self) -> ValidationResult {
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

impl CoreWorkloadType {
    pub fn delete(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonServerType(sing) => sing.delete(),
            CoreWorkloadType::ReplicatedServerType(repl) => repl.delete(),
            CoreWorkloadType::SingletonTaskType(task) => task.delete(),
            CoreWorkloadType::ReplicatedTaskType(task) => task.delete(),
            CoreWorkloadType::ReplicatedWorkerType(task) => task.delete(),
            CoreWorkloadType::SingletonWorkerType(task) => task.delete(),
        }
    }
    pub fn add(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonServerType(sing) => sing.add(),
            CoreWorkloadType::ReplicatedServerType(repl) => repl.add(),
            CoreWorkloadType::SingletonTaskType(task) => task.add(),
            CoreWorkloadType::ReplicatedTaskType(task) => task.add(),
            CoreWorkloadType::ReplicatedWorkerType(task) => task.add(),
            CoreWorkloadType::SingletonWorkerType(task) => task.add(),
        }
    }
    pub fn modify(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonServerType(sing) => sing.modify(),
            CoreWorkloadType::ReplicatedServerType(repl) => repl.modify(),
            CoreWorkloadType::SingletonTaskType(task) => task.modify(),
            CoreWorkloadType::ReplicatedTaskType(task) => task.modify(),
            CoreWorkloadType::ReplicatedWorkerType(task) => task.modify(),
            CoreWorkloadType::SingletonWorkerType(task) => task.modify(),
        }
    }
    pub fn status(&self) -> StatusResult {
        match self {
            CoreWorkloadType::SingletonServerType(sing) => sing.status(),
            CoreWorkloadType::ReplicatedServerType(repl) => repl.status(),
            CoreWorkloadType::SingletonTaskType(task) => task.status(),
            CoreWorkloadType::ReplicatedTaskType(task) => task.status(),
            CoreWorkloadType::ReplicatedWorkerType(task) => task.status(),
            CoreWorkloadType::SingletonWorkerType(task) => task.status(),
        }
    }
    pub fn validate(&self) -> ValidationResult {
        match self {
            CoreWorkloadType::SingletonServerType(sing) => sing.validate(),
            CoreWorkloadType::ReplicatedServerType(repl) => repl.validate(),
            CoreWorkloadType::SingletonTaskType(task) => task.validate(),
            CoreWorkloadType::ReplicatedTaskType(task) => task.validate(),
            CoreWorkloadType::ReplicatedWorkerType(task) => task.validate(),
            CoreWorkloadType::SingletonWorkerType(task) => task.validate(),
        }
    }
}
