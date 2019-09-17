use failure::Error;
use kube::client::APIClient;

use scylla::{instigator::OpResource, lifecycle::Phase};

/// An Instigator in healthscope takes an inbound object and effect health scope
/// Instigators know how to deal with the following operations:
/// - Add
/// - Modify
/// - Delete

#[derive(Clone)]
pub struct Instigator {
    client: APIClient,
    namespace: String,
}

pub type InstigatorResult = Result<(), Error>;

impl Instigator {
    /// Create a new instigator
    ///
    /// An instigator uses the reflector as a cache of Components, and will use the API client
    /// for creating and managing the component implementation.
    pub fn new(client: kube::client::APIClient, namespace: String) -> Self {
        Instigator { client, namespace }
    }

    /// The workhorse for Instigator.
    /// This will execute only Add, Modify, and Delete phases.
    fn exec(&self, event: OpResource, phase: Phase) -> InstigatorResult {
        Ok(())
    }

    /// Create new Kubernetes objects based on this config.
    pub fn add(&self, event: OpResource) -> InstigatorResult {
        self.exec(event, Phase::Add)
    }
    /// Modify existing Kubernetes objects based on config and workload type.
    pub fn modify(&self, event: OpResource) -> InstigatorResult {
        self.exec(event, Phase::Modify)
    }
    /// Delete the Kubernetes objects associated with this config.
    pub fn delete(&self, event: OpResource) -> InstigatorResult {
        self.exec(event, Phase::Delete)
    }
}
