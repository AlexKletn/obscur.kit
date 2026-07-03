//! The replaceable port for opening targets via the shell (spec §6.11).
//!
//! [`ShellProvider`] is the hexagonal port through which the domain asks the
//! platform shell to open a [`ShellTarget`](crate::shell::ShellTarget) — a file,
//! a URI, or a document token — and receives a
//! [`ShellResult`](crate::shell::ShellResult). App/installer responsibilities
//! (file associations, MIME/protocol registration, manifests, single-instance
//! coordination) are out of this port by design (spec §6.11).

use crate::shell::{ShellResult, ShellTarget};

/// The replaceable port for opening targets through the platform shell
/// (spec §6.11).
///
/// An adapter implements this trait once per platform and the domain holds it as
/// `Box<dyn ShellProvider>`. The trait is deliberately **object-safe** — no
/// generic or `Self`-returning methods — so providers can be stored
/// polymorphically.
///
/// Contract: a provider reports honest, typed outcomes and **never panics** on a
/// target it cannot open — it answers [`ShellResult::Unsupported`] (or
/// [`ShellResult::Denied`]) instead.
pub trait ShellProvider {
    /// Opens `target` through the shell and returns the typed outcome
    /// (spec §6.11).
    ///
    /// A provider that cannot open this target returns
    /// [`ShellResult::Unsupported`] rather than panicking.
    fn open(&mut self, target: ShellTarget) -> ShellResult;

    /// Returns whether this provider can open `target` (spec §6.11).
    #[must_use]
    fn can_open(&self, target: &ShellTarget) -> bool;
}

/// A reference, no-op [`ShellProvider`] that opens nothing (spec §6.11).
///
/// `NullShellProvider` honestly reports that it can open no target:
/// [`Self::open`] returns [`ShellResult::Unsupported`] and [`Self::can_open`]
/// returns `false`. Its answers are correct behavior — a documented reference for
/// the honest-capability contract — not a stub waiting to be filled in.
#[derive(Debug, Default, Clone)]
pub struct NullShellProvider;

impl ShellProvider for NullShellProvider {
    fn open(&mut self, _target: ShellTarget) -> ShellResult {
        // Honest: a null provider can open no target.
        ShellResult::Unsupported
    }

    fn can_open(&self, _target: &ShellTarget) -> bool {
        false
    }
}

/// A deterministic, recording [`ShellProvider`] for tests (spec §6.11).
///
/// `RecordingShellProvider` accepts every target: [`Self::open`] pushes the
/// target onto [`Self::opened`] and returns [`ShellResult::Opened`], and
/// [`Self::can_open`] is always `true`. Tests inspect the recorded targets to
/// assert what the domain attempted to open, without touching a real shell.
#[derive(Debug, Default, Clone)]
pub struct RecordingShellProvider {
    opened: Vec<ShellTarget>,
}

impl RecordingShellProvider {
    /// Creates an empty recording provider (spec §6.11).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the targets opened so far, in order (spec §6.11).
    #[must_use]
    pub fn opened(&self) -> &[ShellTarget] {
        &self.opened
    }
}

impl ShellProvider for RecordingShellProvider {
    fn open(&mut self, target: ShellTarget) -> ShellResult {
        self.opened.push(target);
        ShellResult::Opened
    }

    fn can_open(&self, _target: &ShellTarget) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_provider_is_object_safe() {
        let _: Box<dyn ShellProvider> = Box::new(NullShellProvider);
    }

    #[test]
    fn null_provider_opens_nothing() {
        let mut provider = NullShellProvider;
        let target = ShellTarget::uri("https://example.com");
        assert!(!provider.can_open(&target));
        assert_eq!(provider.open(target), ShellResult::Unsupported);
    }

    #[test]
    fn recording_provider_records_and_opens() {
        let mut provider = RecordingShellProvider::new();
        let target = ShellTarget::uri("https://example.com");
        assert!(provider.can_open(&target));
        assert_eq!(provider.open(target.clone()), ShellResult::Opened);
        assert_eq!(provider.open(ShellTarget::document("doc")), ShellResult::Opened);

        assert_eq!(
            provider.opened(),
            &[target, ShellTarget::document("doc")]
        );
    }
}
