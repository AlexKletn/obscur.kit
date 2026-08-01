#![doc = include_str!("shell_provider.doc/_module.md")]

use crate::shell::{ShellResult, ShellTarget};

#[doc = include_str!("shell_provider.doc/ShellProvider.md")]
pub trait ShellProvider {
    #[doc = include_str!("shell_provider.doc/open.md")]
    fn open(&mut self, target: ShellTarget) -> ShellResult;

    #[doc = include_str!("shell_provider.doc/can_open.md")]
    #[must_use]
    fn can_open(&self, target: &ShellTarget) -> bool;
}

#[doc = include_str!("shell_provider.doc/NullShellProvider.md")]
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

#[doc = include_str!("shell_provider.doc/RecordingShellProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct RecordingShellProvider {
    opened: Vec<ShellTarget>,
}

impl RecordingShellProvider {
    #[doc = include_str!("shell_provider.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("shell_provider.doc/opened.md")]
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
        assert_eq!(
            provider.open(ShellTarget::document("doc")),
            ShellResult::Opened
        );

        assert_eq!(provider.opened(), &[target, ShellTarget::document("doc")]);
    }
}
