#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate regex;

pub mod instigator;
pub mod lifecycle;
pub mod schematic;
mod trait_manager;
pub mod workload_type;

#[cfg(test)]
mod instigator_test;
#[cfg(test)]
mod lifecycle_test;
#[cfg(test)]
mod workload_type_test;
