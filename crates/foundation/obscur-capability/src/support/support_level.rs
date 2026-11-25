//! How well a capability is supported on the current platform.

/// The level at which a platform operation is available (spec §6.2, §8.58).
/// Unsupported operations return a level, never a panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportLevel {
    /// Fully supported natively.
    Native,
    /// Supported through emulation with possible differences.
    Emulated,
    /// Supported with documented restrictions.
    Limited,
    /// Not available on this platform/configuration.
    Unavailable,
}
