use super::{ShellProvider, ShellResult, ShellTarget};

#[doc = include_str!("../../docs/api/shell/shell_provider.doc/NullShellProvider.md")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_provider_opens_nothing() {
        let mut provider = NullShellProvider;
        let target = ShellTarget::uri("https://example.com");
        assert!(!provider.can_open(&target));
        assert_eq!(provider.open(target), ShellResult::Unsupported);
    }
}
