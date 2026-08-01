#![doc = include_str!("permission_duration.doc/_module.md")]

#[doc = include_str!("permission_duration.doc/PermissionDuration.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDuration {
    #[doc = include_str!("permission_duration.doc/OneShot.md")]
    OneShot,
    #[doc = include_str!("permission_duration.doc/Session.md")]
    Session,
    #[doc = include_str!("permission_duration.doc/Persistent.md")]
    Persistent,
}
