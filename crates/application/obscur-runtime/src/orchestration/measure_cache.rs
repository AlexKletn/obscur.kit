//! The §4.2 measure cache: [`MeasureCache`].

use std::collections::HashMap;

use obscur_geometry::{BoxConstraints, Size};
use obscur_identity::{FastSet, NodeId};

#[doc = include_str!("../../docs/api/orchestration/measure_cache.doc/MeasureCache.md")]
#[derive(Debug, Default)]
pub struct MeasureCache {
    /// The memoized sizes.
    entries: std::cell::RefCell<HashMap<MeasureKey, Size>>,
    /// The nodes whose size the current pass actually computed (one entry per
    /// cache miss, so a node measured under two distinct constraints appears
    /// twice). Reset at the top of each pass.
    measured: std::cell::RefCell<Vec<NodeId>>,
    /// First and last baseline results cached with the same constraints as the measurement.
    baselines: std::cell::RefCell<HashMap<(MeasureKey, bool), Option<f32>>>,
    depth_limit_node: std::cell::Cell<Option<NodeId>>,
    grid_limited_item: std::cell::Cell<Option<NodeId>>,
    text_timing: std::cell::Cell<Option<(usize, std::time::Duration)>>,
}

/// The node, the four range bounds, and the two percentage-basis components — each
/// `f32` as its exact bit pattern. See [`measure_key`].
pub(super) type MeasureKey = (NodeId, u32, u32, u32, u32, u32, u32);

impl MeasureCache {
    /// Creates an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The memoized size of `key`, if any.
    pub(super) fn get(&self, key: &MeasureKey) -> Option<Size> {
        self.entries.borrow().get(key).copied()
    }

    /// Memoizes `size` for `key` and records that `node` was actually measured.
    pub(super) fn record(&self, key: MeasureKey, node: NodeId, size: Size) {
        self.entries.borrow_mut().insert(key, size);
        self.measured.borrow_mut().push(node);
    }

    /// Returns a cached answer, including a measured absence of a baseline.
    pub(super) fn baseline(&self, key: MeasureKey, last: bool) -> Option<Option<f32>> {
        self.baselines.borrow().get(&(key, last)).copied()
    }

    /// Stores the requested baseline answer for a measured node and constraints.
    pub(super) fn record_baseline(&self, key: MeasureKey, last: bool, baseline: Option<f32>) {
        self.baselines.borrow_mut().insert((key, last), baseline);
    }

    pub(crate) fn begin_text_timing(&self) {
        self.text_timing.set(Some((0, std::time::Duration::ZERO)));
    }

    pub(crate) fn end_text_timing(&self) -> (usize, std::time::Duration) {
        self.text_timing.take().unwrap_or_default()
    }

    pub(super) fn start_text_measure(&self) -> Option<std::time::Instant> {
        self.text_timing.get().map(|_| std::time::Instant::now())
    }

    pub(super) fn finish_text_measure(&self, started: std::time::Instant) {
        if let Some((count, time)) = self.text_timing.get() {
            self.text_timing
                .set(Some((count + 1, time + started.elapsed())));
        }
    }

    /// Starts a pass: forgets what the previous pass measured, keeping the
    /// memoized sizes.
    pub(super) fn begin_pass(&self) {
        self.measured.borrow_mut().clear();
        self.depth_limit_node.set(None);
        self.grid_limited_item.set(None);
    }

    /// Records the first node omitted by this layout pass.
    pub(super) fn record_depth_limit(&self, node: NodeId) {
        if self.depth_limit_node.get().is_none() {
            self.depth_limit_node.set(Some(node));
        }
    }

    /// The first depth-limit omission in the current pass, if any.
    pub(crate) fn depth_limit_node(&self) -> Option<NodeId> {
        self.depth_limit_node.get()
    }

    /// Records the first grid item whose placement hit a limit in this layout pass.
    pub(super) fn record_grid_limit(&self, item: Option<NodeId>) {
        if self.grid_limited_item.get().is_none() {
            self.grid_limited_item.set(item);
        }
    }

    /// The first grid item whose placement exceeded the implicit grid limit in this pass.
    pub(crate) fn grid_limited_item(&self) -> Option<NodeId> {
        self.grid_limited_item.get()
    }

    /// Forgets every memoized size.
    pub fn clear(&self) {
        self.entries.borrow_mut().clear();
        self.baselines.borrow_mut().clear();
        self.begin_pass();
    }

    /// Drops every memoized size of every node in `nodes`, keeping the rest.
    pub fn evict(&self, nodes: &FastSet<NodeId>) {
        self.entries
            .borrow_mut()
            .retain(|&(node, ..), _| !nodes.contains(&node));
        self.baselines
            .borrow_mut()
            .retain(|&((node, ..), _), _| !nodes.contains(&node));
    }

    /// Whether the last pass **computed** `node`'s size, as opposed to serving it
    /// from a retained entry or never asking for it at all.
    #[must_use]
    pub fn measured(&self, node: NodeId) -> bool {
        self.measured.borrow().contains(&node)
    }

    /// How many measures the last pass computed (cache misses, not distinct
    /// nodes).
    #[must_use]
    pub fn measured_count(&self) -> usize {
        self.measured.borrow().len()
    }

    /// How many `(node, constraints)` sizes are memoized.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.borrow().len()
    }

    /// Whether nothing is memoized.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.borrow().is_empty()
    }
}

#[doc = include_str!("../../docs/api/orchestration/measure_cache.doc/measure_key.md")]
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
