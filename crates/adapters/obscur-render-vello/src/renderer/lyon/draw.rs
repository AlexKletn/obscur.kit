use std::ops::Range;

use super::operation::Operation;

pub(super) struct Draw {
    pub(super) operation: Operation,
    pub(super) depth: u32,
    pub(super) indices: Range<u32>,
}
