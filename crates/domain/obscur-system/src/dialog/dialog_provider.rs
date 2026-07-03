//! The replaceable port for presenting dialogs (spec §6.9 — dialog providers).
//!
//! [`DialogProvider`] is the hexagonal port through which the domain asks the
//! platform to present a [`DialogRequest`](crate::dialog::DialogRequest) and
//! receives a [`DialogResult`](crate::dialog::DialogResult). A concrete provider
//! may be native, an external portal, or kit-rendered (spec §6.9), and providers
//! may be composed into a chain.
//!
//! ## Provider chains and loop detection (spec §6.9)
//!
//! Because a kit-rendered system dialog can itself be *implemented over* a
//! `DialogProvider`, composing providers risks a provider re-entering itself and
//! recursing forever. The port surfaces [`DialogProvider::origin`] — a stable
//! identifier — precisely so a composing caller can record the origins already on
//! the chain and refuse to re-enter one. Loop detection is therefore the
//! *caller's* responsibility, using `origin()`; this pure port models the
//! identity but performs no runtime recursion of its own.

use std::collections::VecDeque;

use obscur_capability::SupportLevel;

use crate::dialog::{DialogRequest, DialogResult};

/// The replaceable port for presenting dialogs to the user (spec §6.9).
///
/// An adapter implements this trait once per presentation strategy (native,
/// portal-external, or kit-rendered) and the domain holds it as
/// `Box<dyn DialogProvider>`. The trait is deliberately **object-safe** — no
/// generic or `Self`-returning methods — so providers can be stored and chained
/// polymorphically.
///
/// Contract: a provider reports honest, typed outcomes and **never panics** on an
/// unsupported request — it answers [`DialogResult::Unsupported`] and reports
/// [`SupportLevel::Unavailable`] from [`Self::supports`] instead. See the module
/// docs for how [`Self::origin`] drives §6.9 chain loop detection.
pub trait DialogProvider {
    /// Returns a stable identifier for this provider (spec §6.9).
    ///
    /// A composing caller records the origins already on a provider chain and
    /// refuses to re-enter one, preventing a kit-rendered dialog implemented over
    /// this provider from recursively re-entering it.
    fn origin(&self) -> &str;

    /// Presents `request` and returns the user's typed outcome (spec §6.9).
    ///
    /// A provider that cannot present this request returns
    /// [`DialogResult::Unsupported`] rather than panicking.
    fn present(&mut self, request: DialogRequest) -> DialogResult;

    /// Reports the honest [`SupportLevel`] at which this provider can satisfy
    /// `request` (spec §6.9).
    #[must_use]
    fn supports(&self, request: &DialogRequest) -> SupportLevel;
}

/// A reference, no-op [`DialogProvider`] that presents nothing (spec §6.9).
///
/// `NullDialogProvider` honestly reports that it can present no dialog:
/// [`Self::origin`] is `"null"`, [`Self::present`] returns
/// [`DialogResult::Unsupported`], and [`Self::supports`] returns
/// [`SupportLevel::Unavailable`] for every request. Its "unsupported" answers are
/// correct behavior — a documented reference for the honest-capability contract —
/// not a stub waiting to be filled in.
#[derive(Debug, Default, Clone)]
pub struct NullDialogProvider;

impl DialogProvider for NullDialogProvider {
    fn origin(&self) -> &'static str {
        "null"
    }

    fn present(&mut self, _request: DialogRequest) -> DialogResult {
        // Honest: a null provider can present no dialog.
        DialogResult::Unsupported
    }

    fn supports(&self, _request: &DialogRequest) -> SupportLevel {
        SupportLevel::Unavailable
    }
}

/// A deterministic, scripted [`DialogProvider`] for tests (spec §6.9).
///
/// `ScriptedDialogProvider` answers each [`Self::present`] call with the next
/// result queued by [`Self::push_result`], in FIFO order, falling back to
/// [`DialogResult::Dismissed`] once the queue is empty. Its [`Self::origin`] is
/// `"scripted"` and it reports [`SupportLevel::Emulated`], letting tests drive
/// domain logic through the port without any real platform dialog.
#[derive(Debug, Default, Clone)]
pub struct ScriptedDialogProvider {
    queue: VecDeque<DialogResult>,
}

impl ScriptedDialogProvider {
    /// Creates an empty scripted provider (spec §6.9).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueues `result` to be returned by a later [`Self::present`] call
    /// (spec §6.9).
    pub fn push_result(&mut self, result: DialogResult) {
        self.queue.push_back(result);
    }
}

impl DialogProvider for ScriptedDialogProvider {
    fn origin(&self) -> &'static str {
        "scripted"
    }

    fn present(&mut self, _request: DialogRequest) -> DialogResult {
        self.queue.pop_front().unwrap_or(DialogResult::Dismissed)
    }

    fn supports(&self, _request: &DialogRequest) -> SupportLevel {
        SupportLevel::Emulated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_provider_is_object_safe() {
        let _: Box<dyn DialogProvider> = Box::new(NullDialogProvider);
    }

    #[test]
    fn null_provider_is_honestly_unsupported() {
        let mut provider = NullDialogProvider;
        assert_eq!(provider.origin(), "null");
        assert_eq!(
            provider.present(DialogRequest::open_file()),
            DialogResult::Unsupported
        );
        assert_eq!(
            provider.supports(&DialogRequest::open_file()),
            SupportLevel::Unavailable
        );
    }

    #[test]
    fn scripted_provider_pops_in_fifo_order_then_dismisses() {
        let mut provider = ScriptedDialogProvider::new();
        assert_eq!(provider.origin(), "scripted");
        assert_eq!(
            provider.supports(&DialogRequest::pick_folder()),
            SupportLevel::Emulated
        );

        provider.push_result(DialogResult::Confirmed(true));
        provider.push_result(DialogResult::Denied);

        assert_eq!(
            provider.present(DialogRequest::confirm("t", "b")),
            DialogResult::Confirmed(true)
        );
        assert_eq!(
            provider.present(DialogRequest::confirm("t", "b")),
            DialogResult::Denied
        );
        // Drained: falls back to Dismissed.
        assert_eq!(
            provider.present(DialogRequest::confirm("t", "b")),
            DialogResult::Dismissed
        );
    }
}
