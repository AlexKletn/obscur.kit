mod shell_provider;
pub use shell_provider::{NullShellProvider, RecordingShellProvider, ShellProvider};
mod shell_result;
pub use shell_result::ShellResult;
mod shell_target;
pub use shell_target::{ActivationKind, ShellTarget};
