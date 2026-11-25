//! Process/IPC `PluginExecutor` adapter.

/// Running a plugin in a separate process.
pub mod executor;
/// How the host and that process exchange messages.
pub mod transport;
