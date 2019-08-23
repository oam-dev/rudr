#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;

pub mod instigator;
pub mod lifecycle;
pub mod schematic;
pub mod workload_type;

#[cfg(test)]
mod instigator_test;
#[cfg(test)]
mod lifecycle_test;
#[cfg(test)]
mod workload_type_test;
