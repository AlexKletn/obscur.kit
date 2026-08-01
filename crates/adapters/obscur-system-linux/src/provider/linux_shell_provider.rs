#![doc = include_str!("linux_shell_provider.doc/_module.md")]

use std::process::Command;

use obscur_system::shell::{ShellProvider, ShellResult, ShellTarget};

#[doc = include_str!("linux_shell_provider.doc/LinuxShellProvider.md")]
#[derive(Debug, Clone)]
pub struct LinuxShellProvider {
    #[doc = include_str!("linux_shell_provider.doc/opener.md")]
    opener: Box<str>,
}

impl LinuxShellProvider {
    #[doc = include_str!("linux_shell_provider.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            opener: Box::from("xdg-open"),
        }
    }

    #[doc = include_str!("linux_shell_provider.doc/with_opener.md")]
    #[must_use]
    pub fn with_opener(opener: impl Into<Box<str>>) -> Self {
        Self {
            opener: opener.into(),
        }
    }

    #[doc = include_str!("linux_shell_provider.doc/opener_2.md")]
    #[must_use]
    pub fn opener(&self) -> &str {
        &self.opener
    }
}

impl Default for LinuxShellProvider {
    #[doc = include_str!("linux_shell_provider.doc/default.md")]
    fn default() -> Self {
        Self::new()
    }
}

impl ShellProvider for LinuxShellProvider {
    fn open(&mut self, target: ShellTarget) -> ShellResult {
        if !tool_available(&self.opener) {
            // No opener on PATH: honestly recoverable-unsupported.
            return ShellResult::Unsupported;
        }
        let arg = target_arg(&target);
        match Command::new(&*self.opener).arg(arg).spawn() {
            // Fire-and-forget: a successful spawn means the request was handed to
            // the desktop. The real application launch is asynchronous — we do not
            // wait on the child (xdg-open detaches).
            Ok(_child) => ShellResult::Opened,
            Err(io) => ShellResult::Failed(io.to_string().into()),
        }
    }

    fn can_open(&self, _target: &ShellTarget) -> bool {
        // Every target kind is openable by xdg-open when the opener exists; the
        // only thing that gates capability is whether the tool is on PATH.
        tool_available(&self.opener)
    }
}

#[doc = include_str!("linux_shell_provider.doc/target_arg.md")]
fn target_arg(target: &ShellTarget) -> String {
    match target {
        ShellTarget::Uri(uri) => uri.to_string(),
        ShellTarget::Document(token) => token.to_string(),
        ShellTarget::File(handle) => handle
            .path_hint()
            .unwrap_or_else(|| handle.token())
            .to_string(),
    }
}

#[doc = include_str!("linux_shell_provider.doc/tool_available.md")]
fn tool_available(cmd: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    path.to_string_lossy()
        .split(':')
        .filter(|dir| !dir.is_empty())
        .any(|dir| std::fs::metadata(std::path::Path::new(dir).join(cmd)).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_system::document::{DocumentHandle, DocumentMetadata, DocumentPermission};

    #[test]
    fn tool_available_finds_sh_and_rejects_nonsense() {
        assert!(tool_available("sh"));
        assert!(!tool_available("definitely-not-a-real-tool-xyz123"));
    }

    #[test]
    fn default_opener_is_xdg_open() {
        assert_eq!(LinuxShellProvider::new().opener(), "xdg-open");
        assert_eq!(LinuxShellProvider::default().opener(), "xdg-open");
    }

    #[test]
    fn missing_opener_cannot_open_and_returns_unsupported() {
        let mut provider = LinuxShellProvider::with_opener("definitely-not-a-real-tool-xyz123");
        let target = ShellTarget::uri("https://example.com");
        assert!(!provider.can_open(&target));
        assert_eq!(provider.open(target), ShellResult::Unsupported);
    }

    #[test]
    fn target_arg_extracts_uri_document_and_file() {
        assert_eq!(
            target_arg(&ShellTarget::uri("https://example.com")),
            "https://example.com"
        );
        assert_eq!(target_arg(&ShellTarget::document("doc-token")), "doc-token");

        let hinted = DocumentHandle::new(
            "tok",
            DocumentMetadata::new("f"),
            DocumentPermission::read_only(),
        )
        .with_path_hint("/home/user/file.txt");
        assert_eq!(
            target_arg(&ShellTarget::file(hinted)),
            "/home/user/file.txt"
        );

        let no_hint = DocumentHandle::new(
            "tok-only",
            DocumentMetadata::new("f"),
            DocumentPermission::read_only(),
        );
        assert_eq!(target_arg(&ShellTarget::file(no_hint)), "tok-only");
    }
}
