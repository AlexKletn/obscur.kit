//! The scope at which a capability is issued.

/// The lifetime/binding scope of an issued capability (spec §8.64).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityScope {
    /// Bound to the whole process.
    Process,
    /// Bound to one host application.
    Application,
    /// Bound to one UI runtime.
    Runtime,
    /// Bound to one contribution instance.
    Instance,
}
