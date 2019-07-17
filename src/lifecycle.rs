/// Phase describes the lifecycle phase for an operation.
///
/// The order of operations is this:
///
/// ADD
/// - Kubernetes Add 
///   - Configuration
/// - PreAdd (traits only): Before components are added
///   - Component Configuration
///   - Traits
/// - Add: Resources added and initialized
///   - Components
///   - Traits
/// 
/// MODIFY
/// - Kubernetes Update
///   - Configuration
/// - PreModify (traits only): Before components are modified
///   - Traits
/// - Modify: Resources are modified
///   - Components
///   - Traits
/// 
/// DELETE
/// - Kubernetes Delete
///   - Configuration
/// - PreDelete:
///   - Traits
/// - Delete:
///   - Components
///   - Traits
/// 
/// Note that in deletion operations, Kubernetes will delete by owner reference before PreDelete. This means
/// that the components will likely be unavailable by the time PreDelete fires. It is only guaranteed to fire
/// before the component's Delete operation is fired.
#[derive(Clone, Debug)]
pub enum Phase {
    // PreAdd happens before resources are added
    PreAdd,
    Add,
    PreModify,
    Modify,
    PreDelete,
    Delete,
}