#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate k8s_openapi;
#[macro_use]
extern crate log;
extern crate env_logger;

pub mod instigator;
pub mod schematic;
pub mod workload_type;

#[cfg(test)]
mod workload_type_test;
