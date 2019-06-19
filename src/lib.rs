#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate k8s_openapi;

pub mod instigator;
pub mod schematic;
pub mod workload_type;

#[cfg(test)]
mod workload_type_test;
