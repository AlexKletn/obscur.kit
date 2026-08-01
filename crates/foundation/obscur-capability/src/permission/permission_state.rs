#![doc = include_str!("permission_state.doc/_module.md")]

#[doc = include_str!("permission_state.doc/PermissionState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    #[doc = include_str!("permission_state.doc/Granted.md")]
    Granted,
    #[doc = include_str!("permission_state.doc/Denied.md")]
    Denied,
    #[doc = include_str!("permission_state.doc/Prompt.md")]
    Prompt,
    #[doc = include_str!("permission_state.doc/Unsupported.md")]
    Unsupported,
}
