#![doc = include_str!("../../docs/api/shell/shell_provider.doc/_module.md")]

use super::{ShellResult, ShellTarget};

#[doc = include_str!("../../docs/api/shell/shell_provider.doc/ShellProvider.md")]
pub trait ShellProvider {
    #[doc = include_str!("../../docs/api/shell/shell_provider.doc/open.md")]
    fn open(&mut self, target: ShellTarget) -> ShellResult;

    /// Returns whether this provider can open `target` (spec §6.11).
    #[must_use]
    fn can_open(&self, target: &ShellTarget) -> bool;
}

#[cfg(test)]
mod tests {
    use super::super::NullShellProvider;
    use super::*;

    #[test]
    fn shell_provider_is_object_safe() {
        let _: Box<dyn ShellProvider> = Box::new(NullShellProvider);
    }
}
