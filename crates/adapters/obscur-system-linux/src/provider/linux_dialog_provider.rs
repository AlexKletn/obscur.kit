#![doc = include_str!("../../docs/api/provider/linux_dialog_provider.doc/_module.md")]

use std::process::{Command, Output};

use obscur_capability::SupportLevel;
use obscur_system::dialog::{DialogProvider, DialogRequest, DialogResult, MessageLevel};
use obscur_system::document::{
    DirectoryHandle, DocumentHandle, DocumentMetadata, DocumentPermission,
};

#[doc = include_str!("../../docs/api/provider/linux_dialog_provider.doc/LinuxDialogProvider.md")]
#[derive(Debug, Clone)]
pub struct LinuxDialogProvider {
    /// The dialog tool to invoke, e.g. `"zenity"`.
    tool: Box<str>,
}

impl LinuxDialogProvider {
    /// Creates a provider that presents dialogs with `zenity` (spec §6.9).
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool: Box::from("zenity"),
        }
    }

    /// Creates a provider that presents dialogs with a custom `tool` command
    /// (spec §6.9).
    #[must_use]
    pub fn with_tool(tool: impl Into<Box<str>>) -> Self {
        Self { tool: tool.into() }
    }

    /// Returns the dialog tool this provider invokes (spec §6.9).
    #[must_use]
    pub fn tool(&self) -> &str {
        &self.tool
    }

    /// Runs the configured tool with `args`, capturing its output (spec §6.9).
    fn run_zenity(&self, args: &[&str]) -> std::io::Result<Output> {
        Command::new(&*self.tool).args(args).output()
    }
}

impl Default for LinuxDialogProvider {
    /// The default provider presents dialogs with `zenity`.
    fn default() -> Self {
        Self::new()
    }
}

impl DialogProvider for LinuxDialogProvider {
    fn origin(&self) -> &'static str {
        "linux-zenity"
    }

    fn present(&mut self, request: DialogRequest) -> DialogResult {
        if !tool_available(&self.tool) {
            // No zenity on PATH: honestly recoverable-unsupported.
            return DialogResult::Unsupported;
        }

        let built = build_args(&request);
        let args: Vec<&str> = built.iter().map(String::as_str).collect();
        // A spawn/IO error has no dedicated result; report it best-effort as a
        // dismissal (as if the user closed the dialog without choosing).
        let Ok(output) = self.run_zenity(&args) else {
            return DialogResult::Dismissed;
        };
        let stdout = String::from_utf8_lossy(&output.stdout);

        match request {
            DialogRequest::OpenFile { .. } => {
                if output.status.success() {
                    let files = parse_paths(&stdout)
                        .into_iter()
                        .map(|path| handle_from_path(&path, DocumentPermission::read_only()))
                        .collect();
                    DialogResult::Files(files)
                } else {
                    // Non-zero exit: the user cancelled.
                    DialogResult::Dismissed
                }
            }
            DialogRequest::SaveFile { .. } => match parse_paths(&stdout).into_iter().next() {
                Some(path) if output.status.success() => {
                    DialogResult::Saved(handle_from_path(&path, DocumentPermission::read_write()))
                }
                _ => DialogResult::Dismissed,
            },
            DialogRequest::PickFolder => match parse_paths(&stdout).into_iter().next() {
                Some(path) if output.status.success() => {
                    DialogResult::Folder(directory_from_path(&path))
                }
                _ => DialogResult::Dismissed,
            },
            DialogRequest::Message { .. } => {
                // Informational: acknowledged on success, dismissed otherwise.
                if output.status.success() {
                    DialogResult::Confirmed(true)
                } else {
                    DialogResult::Dismissed
                }
            }
            DialogRequest::Confirm { .. } => {
                // zenity --question: exit 0 == yes, exit 1 == no.
                DialogResult::Confirmed(output.status.success())
            }
        }
    }

    fn supports(&self, _request: &DialogRequest) -> SupportLevel {
        if tool_available(&self.tool) {
            SupportLevel::Native
        } else {
            SupportLevel::Unavailable
        }
    }
}

#[doc = include_str!("../../docs/api/provider/linux_dialog_provider.doc/build_args.md")]
#[must_use]
fn build_args(request: &DialogRequest) -> Vec<String> {
    match request {
        DialogRequest::OpenFile { allow_multiple, .. } => {
            let mut args = vec!["--file-selection".to_string()];
            if *allow_multiple {
                args.push("--multiple".to_string());
            }
            args
        }
        DialogRequest::SaveFile { suggested_name, .. } => vec![
            "--file-selection".to_string(),
            "--save".to_string(),
            "--confirm-overwrite".to_string(),
            "--filename".to_string(),
            suggested_name.to_string(),
        ],
        DialogRequest::PickFolder => {
            vec!["--file-selection".to_string(), "--directory".to_string()]
        }
        DialogRequest::Message { title, body, level } => vec![
            message_flag(*level).to_string(),
            "--title".to_string(),
            title.to_string(),
            "--text".to_string(),
            body.to_string(),
        ],
        DialogRequest::Confirm { title, body } => vec![
            "--question".to_string(),
            "--title".to_string(),
            title.to_string(),
            "--text".to_string(),
            body.to_string(),
        ],
    }
}

/// Maps a [`MessageLevel`] to the corresponding `zenity` flag (spec §6.9).
fn message_flag(level: MessageLevel) -> &'static str {
    match level {
        MessageLevel::Info => "--info",
        MessageLevel::Warning => "--warning",
        MessageLevel::Error => "--error",
    }
}

#[doc = include_str!("../../docs/api/provider/linux_dialog_provider.doc/parse_paths.md")]
fn parse_paths(stdout: &str) -> Vec<String> {
    stdout
        .split(['\n', '|'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[doc = include_str!("../../docs/api/provider/linux_dialog_provider.doc/handle_from_path.md")]
fn handle_from_path(path: &str, permission: DocumentPermission) -> DocumentHandle {
    DocumentHandle::new(path, DocumentMetadata::new(basename(path)), permission)
        .with_path_hint(path)
}

/// Builds a read-only [`DirectoryHandle`] from a picked directory `path`
/// (spec §6.10).
fn directory_from_path(path: &str) -> DirectoryHandle {
    DirectoryHandle::new(
        path,
        DocumentMetadata::new(basename(path)).directory(true),
        DocumentPermission::read_only(),
    )
    .with_path_hint(path)
}

/// Returns the last path component of `path` for display, or the whole string
/// when there is no separator.
fn basename(path: &str) -> &str {
    path.rsplit('/').find(|s| !s.is_empty()).unwrap_or(path)
}

#[doc = include_str!("../../docs/api/provider/linux_dialog_provider.doc/tool_available.md")]
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
    use obscur_system::dialog::FileFilter;

    #[test]
    fn origin_is_linux_zenity() {
        assert_eq!(LinuxDialogProvider::new().origin(), "linux-zenity");
    }

    #[test]
    fn default_tool_is_zenity() {
        assert_eq!(LinuxDialogProvider::new().tool(), "zenity");
        assert_eq!(LinuxDialogProvider::default().tool(), "zenity");
    }

    #[test]
    fn missing_tool_is_unavailable_and_unsupported() {
        let mut provider = LinuxDialogProvider::with_tool("definitely-not-a-real-tool-xyz123");
        assert_eq!(
            provider.supports(&DialogRequest::open_file()),
            SupportLevel::Unavailable
        );
        assert_eq!(
            provider.present(DialogRequest::open_file()),
            DialogResult::Unsupported
        );
    }

    #[test]
    fn build_args_open_file_single_and_multiple() {
        assert_eq!(
            build_args(&DialogRequest::open_file()),
            ["--file-selection"]
        );
        assert_eq!(
            build_args(&DialogRequest::open_files()),
            ["--file-selection", "--multiple"]
        );
        // Filters do not change the argv in this minimal mapping.
        let filtered = DialogRequest::OpenFile {
            filters: vec![FileFilter::new("Images", ["png"])],
            allow_multiple: false,
        };
        assert_eq!(build_args(&filtered), ["--file-selection"]);
    }

    #[test]
    fn build_args_save_file_sets_filename() {
        assert_eq!(
            build_args(&DialogRequest::save_file("draft.txt")),
            [
                "--file-selection",
                "--save",
                "--confirm-overwrite",
                "--filename",
                "draft.txt",
            ]
        );
    }

    #[test]
    fn build_args_pick_folder() {
        assert_eq!(
            build_args(&DialogRequest::pick_folder()),
            ["--file-selection", "--directory"]
        );
    }

    #[test]
    fn build_args_message_maps_level_to_flag() {
        assert_eq!(
            build_args(&DialogRequest::message("T", "B", MessageLevel::Info)),
            ["--info", "--title", "T", "--text", "B"]
        );
        assert_eq!(
            build_args(&DialogRequest::message("T", "B", MessageLevel::Warning))[0],
            "--warning"
        );
        assert_eq!(
            build_args(&DialogRequest::message("T", "B", MessageLevel::Error))[0],
            "--error"
        );
    }

    #[test]
    fn build_args_confirm_uses_question() {
        assert_eq!(
            build_args(&DialogRequest::confirm("Delete?", "Sure?")),
            ["--question", "--title", "Delete?", "--text", "Sure?"]
        );
    }

    #[test]
    fn parse_paths_splits_on_newline_and_pipe() {
        assert_eq!(
            parse_paths("/a/one.txt|/b/two.txt\n"),
            ["/a/one.txt", "/b/two.txt"]
        );
        assert_eq!(parse_paths("  \n"), Vec::<String>::new());
    }

    #[test]
    fn handle_from_path_uses_path_as_token_hint_and_basename() {
        let handle = handle_from_path("/home/user/Notes.txt", DocumentPermission::read_only());
        assert_eq!(handle.token(), "/home/user/Notes.txt");
        assert_eq!(handle.path_hint(), Some("/home/user/Notes.txt"));
        assert_eq!(handle.name(), "Notes.txt");
        assert!(!handle.can_write());
    }

    #[test]
    fn directory_from_path_is_directory_and_read_only() {
        let dir = directory_from_path("/home/user/Photos");
        assert_eq!(dir.token(), "/home/user/Photos");
        assert_eq!(dir.name(), "Photos");
        assert!(dir.metadata().is_directory());
        assert!(!dir.can_write());
    }
}
