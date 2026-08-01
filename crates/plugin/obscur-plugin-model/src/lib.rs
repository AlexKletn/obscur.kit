#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

#[doc = include_str!("lib.doc/capability.md")]
pub mod capability;
#[doc = include_str!("lib.doc/contribution.md")]
pub mod contribution;
#[doc = include_str!("lib.doc/identity.md")]
pub mod identity;
#[doc = include_str!("lib.doc/lifecycle.md")]
pub mod lifecycle;
#[doc = include_str!("lib.doc/manifest.md")]
pub mod manifest;
#[doc = include_str!("lib.doc/message.md")]
pub mod message;
#[doc = include_str!("lib.doc/service.md")]
pub mod service;
#[doc = include_str!("lib.doc/settings.md")]
pub mod settings;
#[doc = include_str!("lib.doc/state.md")]
pub mod state;
