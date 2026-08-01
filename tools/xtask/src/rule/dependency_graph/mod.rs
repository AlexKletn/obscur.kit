#![doc = include_str!("mod.doc/_module.md")]

pub(crate) mod check;
pub(crate) mod crate_node;
pub(crate) mod graph;
pub(crate) mod layer;

pub(crate) use check::check;
