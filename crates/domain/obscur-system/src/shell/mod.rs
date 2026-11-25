//! Shell module.
mod shell_provider;
pub use shell_provider::ShellProvider;
mod null_shell_provider;
pub use null_shell_provider::NullShellProvider;
mod recording_shell_provider;
pub use recording_shell_provider::RecordingShellProvider;
mod shell_result;
pub use shell_result::ShellResult;
mod shell_target;
pub use shell_target::ShellTarget;
mod activation_kind;
pub use activation_kind::ActivationKind;
