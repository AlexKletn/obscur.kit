use super::{ShellProvider, ShellResult, ShellTarget};

#[doc = include_str!("../../docs/api/shell/shell_provider.doc/RecordingShellProvider.md")]
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
