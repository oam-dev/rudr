use std::collections::BTreeMap;

mod service;
pub use crate::workload_type::service::{SingletonService, ReplicatedService};
#[cfg(test)]
mod service_test;

mod task;
pub use crate::workload_type::task::{SingletonTask, ReplicatedTask};
#[cfg(test)]
mod task_test;

mod worker;
pub use crate::workload_type::worker::{Worker, SingletonWorker};

mod workload_builder;

pub const HYDRA_API_VERSION: &'static str = "core.hydra.io/v1alpha1";
/// The fully qualified name of a replicated service.
pub const REPLICATED_SERVICE_NAME: &'static str = "core.hydra.io/v1alpha1.ReplicatedService";
/// The fully qualified name of a singleton.
pub const SINGLETON_NAME: &'static str = "core.hydra.io/v1alpha1.Singleton";

pub const TASK_NAME: &'static str = "core.hydra.io/v1alpha1.Task";
pub const REPLICATED_TASK_NAME: &'static str = "core.hydra.io/v1alpha1.ReplicatedTask";

type InstigatorResult = Result<(), failure::Error>;
pub type ParamMap = BTreeMap<String, serde_json::Value>;

/// KubeName describes anything that can produce its own Kubernetes name.
///
/// Most Kubernetes objects have their own name, and workload types, traits, and
/// other Hydra objects are capable of autogenerating their own names in a
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
}

pub enum CoreWorkloadType {
    SingletonType(SingletonService),
    ReplicatedServiceType(ReplicatedService),
    SingletonTaskType(SingletonTask),
    ReplicatedTaskType(ReplicatedTask),
    WorkerType(Worker),
    SingletonWorkerType(SingletonWorker)
}

impl CoreWorkloadType {
    pub fn delete(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonType(sing) => sing.delete(),
            CoreWorkloadType::ReplicatedServiceType(repl) => repl.delete(),
            CoreWorkloadType::SingletonTaskType(task) => task.delete(),
            CoreWorkloadType::ReplicatedTaskType(task) => task.delete(),
            CoreWorkloadType::WorkerType(task) => task.delete(),
            CoreWorkloadType::SingletonWorkerType(task) => task.delete(),

        }
    }
    pub fn add(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonType(sing) => sing.add(),
            CoreWorkloadType::ReplicatedServiceType(repl) => repl.add(),
            CoreWorkloadType::SingletonTaskType(task) => task.add(),
            CoreWorkloadType::ReplicatedTaskType(task) => task.add(),
            CoreWorkloadType::WorkerType(task) => task.add(),
            CoreWorkloadType::SingletonWorkerType(task) => task.add(),

        }
    }
    pub fn modify(&self) -> InstigatorResult {
        Err(format_err!("modify operation is not implemented"))
    }
}
