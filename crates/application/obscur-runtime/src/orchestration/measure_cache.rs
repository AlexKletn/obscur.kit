#![doc = include_str!("measure_cache.doc/_module.md")]

use std::collections::{HashMap, HashSet};

use obscur_geometry::{BoxConstraints, Size};
use obscur_identity::NodeId;

#[doc = include_str!("measure_cache.doc/MeasureCache.md")]
#[derive(Debug, Default)]
pub struct MeasureCache {
    #[doc = include_str!("measure_cache.doc/entries.md")]
    entries: std::cell::RefCell<HashMap<MeasureKey, Size>>,
    #[doc = include_str!("measure_cache.doc/measured.md")]
    measured: std::cell::RefCell<Vec<NodeId>>,
}

#[doc = include_str!("measure_cache.doc/MeasureKey.md")]
pub(super) type MeasureKey = (NodeId, u32, u32, u32, u32, u32, u32);

impl MeasureCache {
    #[doc = include_str!("measure_cache.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("measure_cache.doc/get.md")]
    pub(super) fn get(&self, key: &MeasureKey) -> Option<Size> {
        self.entries.borrow().get(key).copied()
    }

    #[doc = include_str!("measure_cache.doc/record.md")]
    pub(super) fn record(&self, key: MeasureKey, node: NodeId, size: Size) {
        self.entries.borrow_mut().insert(key, size);
        self.measured.borrow_mut().push(node);
    }

    #[doc = include_str!("measure_cache.doc/begin_pass.md")]
    pub(super) fn begin_pass(&self) {
        self.measured.borrow_mut().clear();
    }

    #[doc = include_str!("measure_cache.doc/clear.md")]
    pub fn clear(&self) {
        self.entries.borrow_mut().clear();
        self.measured.borrow_mut().clear();
    }

    #[doc = include_str!("measure_cache.doc/evict.md")]
    pub fn evict(&self, nodes: &HashSet<NodeId>) {
        self.entries
            .borrow_mut()
            .retain(|&(node, ..), _| !nodes.contains(&node));
    }

    #[doc = include_str!("measure_cache.doc/measured_2.md")]
    #[must_use]
    pub fn measured(&self, node: NodeId) -> bool {
        self.measured.borrow().contains(&node)
    }

    #[doc = include_str!("measure_cache.doc/measured_count.md")]
    #[must_use]
    pub fn measured_count(&self) -> usize {
        self.measured.borrow().len()
    }

    #[doc = include_str!("measure_cache.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.borrow().len()
    }

    #[doc = include_str!("measure_cache.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.borrow().is_empty()
    }
}

#[doc = include_str!("measure_cache.doc/measure_key.md")]
pub(super) fn measure_key(node: NodeId, constraints: BoxConstraints) -> MeasureKey {
    let basis = constraints
        .percent_basis
        .unwrap_or(Size::new(f32::NAN, f32::NAN));
    (
        node,
        constraints.min.width.to_bits(),
        constraints.min.height.to_bits(),
        constraints.max.width.to_bits(),
        constraints.max.height.to_bits(),
        basis.width.to_bits(),
        basis.height.to_bits(),
    )
}
