#[doc = include_str!("../../docs/api/shell/shell_target.doc/ActivationKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationKind {
    /// A file or resolved document handle.
    File,
    /// A conventional URI (an `http`, `https`, `mailto`, or `file` scheme).
    Uri,
    /// A custom/app-scheme URI used as a deep link into the application.
    DeepLink,
}
