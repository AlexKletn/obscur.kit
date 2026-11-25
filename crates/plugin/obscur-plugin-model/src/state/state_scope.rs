//! Scopes for plugin state and cache.

/// Areas of ordinary plugin storage within an installation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StateScope {
    /// State shared within one plugin installation.
    Global,
    /// Durable state of a host-identified contribution instance.
    Instance(Box<str>),
    /// Rebuildable cache of one plugin installation.
    Cache,
}
