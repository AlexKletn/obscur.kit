//! Graph-based dependency architecture checks (spec §3.16, §5.9).

pub(crate) mod check;
pub(crate) mod crate_node;
pub(crate) mod graph;
pub(crate) mod layer;

pub(crate) use check::check;
