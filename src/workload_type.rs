use failure::Error;
use log::info;
use std::collections::BTreeMap;

mod service;
pub use crate::workload_type::service::{ReplicatedService, SingletonService};

mod task;
pub use crate::workload_type::task::{ReplicatedTask, SingletonTask};

mod worker;
pub use crate::workload_type::worker::{ReplicatedWorker, SingletonWorker};

mod workload_builder;
pub use crate::workload_type::workload_builder::WorkloadMetadata;

pub const HYDRA_API_VERSION: &str = "core.hydra.io/v1alpha1";

/// Server is a replicable server
pub const SERVER_NAME: &str = "core.hydra.io/v1alpha1.Server";
/// SingletonService is a kind of Server that can't be replicated
pub const SINGLETON_SERVER_NAME: &str = "core.hydra.io/v1alpha1.SingletonServer";

/// SingletonTask is a task that cannot be replicated
pub const SINGLETON_TASK_NAME: &str = "core.hydra.io/v1alpha1.SingletonTask";
/// Task just means a replicated task which will replace ReplicatedTask
pub const TASK_NAME: &str = "core.hydra.io/v1alpha1.Task";

/// Singleton worker is a Worker that cannot be replicated
pub const SINGLETON_WORKER: &str = "core.hydra.io/v1alpha1.SingletonWorker";
/// Worker is daemon process that does not listen on the network
pub const WORKER_NAME: &str = "core.hydra.io/v1alpha1.Worker";

type InstigatorResult = Result<(), Error>;
type StatusResult = Result<BTreeMap<String, String>, Error>;
pub type ParamMap = BTreeMap<String, serde_json::Value>;

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
    fn add(&self) -> InstigatorResult;
    fn modify(&self) -> InstigatorResult {
        Err(format_err!("Not implemented"))
    }
    fn delete(&self) -> InstigatorResult {
        info!("Workload deleted");
        Ok(())
    }
    fn status(&self) -> StatusResult {
        Err(format_err!("Not implemented"))
    }
}

pub enum CoreWorkloadType {
    SingletonServiceType(SingletonService),
    ReplicatedServiceType(ReplicatedService),
    SingletonTaskType(SingletonTask),
    ReplicatedTaskType(ReplicatedTask),
    ReplicatedWorkerType(ReplicatedWorker),
    SingletonWorkerType(SingletonWorker),
}

impl CoreWorkloadType {
    pub fn delete(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonServiceType(sing) => sing.delete(),
            CoreWorkloadType::ReplicatedServiceType(repl) => repl.delete(),
            CoreWorkloadType::SingletonTaskType(task) => task.delete(),
            CoreWorkloadType::ReplicatedTaskType(task) => task.delete(),
            CoreWorkloadType::ReplicatedWorkerType(task) => task.delete(),
            CoreWorkloadType::SingletonWorkerType(task) => task.delete(),
        }
    }
    pub fn add(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonServiceType(sing) => sing.add(),
            CoreWorkloadType::ReplicatedServiceType(repl) => repl.add(),
            CoreWorkloadType::SingletonTaskType(task) => task.add(),
            CoreWorkloadType::ReplicatedTaskType(task) => task.add(),
            CoreWorkloadType::ReplicatedWorkerType(task) => task.add(),
            CoreWorkloadType::SingletonWorkerType(task) => task.add(),
        }
    }
    pub fn modify(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonServiceType(sing) => sing.modify(),
            CoreWorkloadType::ReplicatedServiceType(repl) => repl.modify(),
            CoreWorkloadType::SingletonTaskType(task) => task.modify(),
            CoreWorkloadType::ReplicatedTaskType(task) => task.modify(),
            CoreWorkloadType::ReplicatedWorkerType(task) => task.modify(),
            CoreWorkloadType::SingletonWorkerType(task) => task.modify(),
        }
    }
    pub fn status(&self) -> StatusResult {
        match self {
            CoreWorkloadType::SingletonServiceType(sing) => sing.status(),
            CoreWorkloadType::ReplicatedServiceType(repl) => repl.status(),
            CoreWorkloadType::SingletonTaskType(task) => task.status(),
            CoreWorkloadType::ReplicatedTaskType(task) => task.status(),
            CoreWorkloadType::ReplicatedWorkerType(task) => task.status(),
            CoreWorkloadType::SingletonWorkerType(task) => task.status(),
        }
    }
}
