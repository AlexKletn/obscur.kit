#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod command;
pub mod event;
pub mod focus;
pub mod gamepad;
pub mod gesture;
pub mod hover;
pub mod pointer;
pub mod selection;
pub mod spatial;
