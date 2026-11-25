//! Post-layout geometry notifications.

use obscur_geometry::{AffineTransform, Point, Rect, Size};
use obscur_identity::NodeId;
use obscur_layout::model::LayoutBox;
use obscur_layout::observer::{
    IntersectionChange, IntersectionObserver, LayoutChange, LayoutObserver, ResizeChange,
    ResizeObserver, TransformChange, TransformObserver, VisibilityChange, VisibilityObserver,
};
use obscur_style::{computed::OverflowAxes, property::layout_keyword::OVERFLOW_VISIBLE};
use obscur_tree::tree::RetainedTree;

use crate::frame::FrameReason;
use crate::orchestration::{ComputedStyles, LayoutBoxes};
use crate::runtime::ancestor_geometry::AncestorGeometry;

use super::UiRuntime;
use super::visible_content_subscription::VisibleContentSubscription;

/// A domain observer paired with its post-layout callback.
pub(super) enum GeometrySubscription {
    Resize(ResizeObserver, Box<dyn FnMut(ResizeChange) -> bool>),
    Layout(LayoutObserver, Box<dyn FnMut(LayoutChange) -> bool>),
    Visibility(VisibilityObserver, Box<dyn FnMut(VisibilityChange) -> bool>),
    Intersection(
        IntersectionObserver,
        Box<dyn FnMut(IntersectionChange) -> bool>,
    ),
    Transform(TransformObserver, Box<dyn FnMut(TransformChange) -> bool>),
}

impl GeometrySubscription {
    fn node(&self) -> NodeId {
        match self {
            Self::Resize(observer, _) => observer.node(),
            Self::Layout(observer, _) => observer.node(),
            Self::Visibility(observer, _) => observer.node(),
            Self::Intersection(observer, _) => observer.node(),
            Self::Transform(observer, _) => observer.node(),
        }
    }

    fn publish(
        &mut self,
        layout: Option<LayoutBox>,
        hidden: bool,
        exposure: impl FnOnce() -> (LayoutBox, Rect),
        transform: impl FnOnce() -> AffineTransform,
    ) -> bool {
        match self {
            Self::Resize(observer, callback) => match layout {
                Some(layout) => observer.observe(layout.size()).is_some_and(callback),
                None => {
                    observer.reset();
                    false
                }
            },
            Self::Layout(observer, callback) => match layout {
                Some(layout) => observer.observe(layout).is_some_and(callback),
                None => {
                    observer.reset();
                    false
                }
            },
            Self::Visibility(observer, callback) => {
                let (target, root) = exposure();
                let target = if !hidden && root.size.width > 0.0 && root.size.height > 0.0 {
                    target
                } else {
                    LayoutBox::new(Rect::default())
                };
                observer.observe(target, Some(root)).is_some_and(callback)
            }
            Self::Intersection(observer, callback) => {
                let (target, root) = exposure();
                observer.observe(target.bounds, root).is_some_and(callback)
            }
            Self::Transform(observer, callback) => match layout {
                Some(_) => observer.observe(transform()).is_some_and(callback),
                None => {
                    observer.reset();
                    false
                }
            },
        }
    }
}

impl UiRuntime {
    /// Observes a live node's committed logical size. The first notification reports
    /// the initial size. Following notifications run when the committed size changes.
    /// Return true from the callback to request a UI commit on the next frame.
    /// Re-registering replaces the callback. Removing the node drops its subscription.
    pub fn observe_resize(
        &mut self,
        node: NodeId,
        callback: impl FnMut(ResizeChange) -> bool + 'static,
    ) -> bool {
        if !self.tree.contains(node) {
            return false;
        }
        self.unobserve_resize(node);
        self.geometry_observers.push(GeometrySubscription::Resize(
            ResizeObserver::new(node),
            Box::new(callback),
        ));
        self.request_frame(FrameReason::UiCommit);
        true
    }

    /// Removes a node's resize subscription, returning whether it existed.
    pub fn unobserve_resize(&mut self, node: NodeId) -> bool {
        let before = self.geometry_observers.len();
        self.geometry_observers.retain(|entry| {
            !matches!(entry, GeometrySubscription::Resize(observer, _) if observer.node() == node)
        });
        self.geometry_observers.len() != before
    }

    /// Observes committed layout boxes in logical layout coordinates. The callback receives the first box and subsequent changes. Return true to request a UI commit on the next frame.
    pub fn observe_layout(
        &mut self,
        node: NodeId,
        callback: impl FnMut(LayoutChange) -> bool + 'static,
    ) -> bool {
        if !self.tree.contains(node) {
            return false;
        }
        self.unobserve_layout(node);
        self.geometry_observers.push(GeometrySubscription::Layout(
            LayoutObserver::new(node),
            Box::new(callback),
        ));
        self.request_frame(FrameReason::UiCommit);
        true
    }

    /// Removes a node’s layout subscription, returning whether it existed.
    pub fn unobserve_layout(&mut self, node: NodeId) -> bool {
        let before = self.geometry_observers.len();
        self.geometry_observers.retain(|entry| {
            !matches!(entry, GeometrySubscription::Layout(observer, _) if observer.node() == node)
        });
        self.geometry_observers.len() != before
    }

    /// Observes whether a node has positive visible layout area in the viewport after overflow clipping and CSS visibility. Return true to request a follow-up UI commit.
    pub fn observe_visibility(
        &mut self,
        node: NodeId,
        callback: impl FnMut(VisibilityChange) -> bool + 'static,
    ) -> bool {
        if !self.tree.contains(node) {
            return false;
        }
        self.unobserve_visibility(node);
        self.geometry_observers
            .push(GeometrySubscription::Visibility(
                VisibilityObserver::new(node),
                Box::new(callback),
            ));
        self.request_frame(FrameReason::UiCommit);
        true
    }

    /// Removes the node’s visibility subscription.
    pub fn unobserve_visibility(&mut self, node: NodeId) -> bool {
        let before = self.geometry_observers.len();
        self.geometry_observers.retain(|entry| {
            !matches!(entry, GeometrySubscription::Visibility(observer, _) if observer.node() == node)
        });
        self.geometry_observers.len() != before
    }

    /// Observes crossings of a visible-area ratio threshold against the viewport and clipping ancestors. Return true to request a follow-up UI commit.
    pub fn observe_intersection(
        &mut self,
        node: NodeId,
        threshold: f32,
        callback: impl FnMut(IntersectionChange) -> bool + 'static,
    ) -> bool {
        if !self.tree.contains(node) || !threshold.is_finite() {
            return false;
        }
        self.unobserve_intersection(node);
        self.geometry_observers
            .push(GeometrySubscription::Intersection(
                IntersectionObserver::new(node, threshold),
                Box::new(callback),
            ));
        self.request_frame(FrameReason::UiCommit);
        true
    }

    /// Removes the node’s intersection subscription.
    pub fn unobserve_intersection(&mut self, node: NodeId) -> bool {
        let before = self.geometry_observers.len();
        self.geometry_observers.retain(|entry| {
            !matches!(entry, GeometrySubscription::Intersection(observer, _) if observer.node() == node)
        });
        self.geometry_observers.len() != before
    }

    /// Reports the composed CSS transform including visual ancestors. Return true to request a follow-up commit.
    pub fn observe_transform(
        &mut self,
        node: NodeId,
        callback: impl FnMut(TransformChange) -> bool + 'static,
    ) -> bool {
        if !self.tree.contains(node) {
            return false;
        }
        self.unobserve_transform(node);
        self.geometry_observers
            .push(GeometrySubscription::Transform(
                TransformObserver::new(node),
                Box::new(callback),
            ));
        self.request_frame(FrameReason::UiCommit);
        true
    }

    /// Removes a node’s transform subscription.
    pub fn unobserve_transform(&mut self, node: NodeId) -> bool {
        let before = self.geometry_observers.len();
        self.geometry_observers.retain(|entry| {
            !matches!(entry, GeometrySubscription::Transform(observer, _) if observer.node() == node)
        });
        self.geometry_observers.len() != before
    }

    /// Observes the visible rectangle in logical content coordinates after each committed frame. The next frame reports the initial rectangle. Later frames report it only when it changes. Returning true schedules a UI commit. A new registration replaces the previous observer.
    pub fn observe_visible_content(&mut self, callback: impl FnMut(Rect) -> bool + 'static) {
        self.visible_content_observer = Some(VisibleContentSubscription {
            last: None,
            callback: Box::new(callback),
        });
        self.request_frame(FrameReason::UiCommit);
    }

    /// Removes the current visible-content observer and reports whether one was installed.
    pub fn unobserve_visible_content(&mut self) -> bool {
        self.visible_content_observer.take().is_some()
    }

    /// Publishes change notifications after layout and coalesces follow-up commits.
    pub(super) fn publish_geometry_observations(&mut self) {
        let viewport = self.surface.visible_content_rect();
        let mut request_commit = self
            .visible_content_observer
            .as_mut()
            .is_some_and(|observer| observer.publish(viewport));
        let projected = self.scroll.projection(&self.boxes);
        let mut transforms = AncestorGeometry::new(0);
        self.geometry_observers.retain_mut(|entry| {
            let node = entry.node();
            if !self.tree.contains(node) {
                return false;
            }
            request_commit |= entry.publish(
                self.boxes.get(&node).copied(),
                self.computed
                    .get(&node)
                    .is_some_and(|style| style.is_visibility_hidden()),
                || visible_layout(&self.tree, projected, &self.computed, node, viewport),
                || {
                    transforms
                        .for_node(&self.tree, &self.computed, &self.boxes, node)
                        .1
                        .unwrap_or(AffineTransform::IDENTITY)
                },
            );
            true
        });
        if request_commit {
            self.request_frame(FrameReason::UiCommit);
        }
    }
}

/// Returns the projected target and the rectangular intersection of its ancestor clips with the viewport.
fn visible_layout(
    tree: &RetainedTree,
    boxes: &LayoutBoxes,
    styles: &ComputedStyles,
    node: NodeId,
    viewport: Rect,
) -> (LayoutBox, Rect) {
    let target = boxes
        .get(&node)
        .copied()
        .unwrap_or_else(|| LayoutBox::new(Rect::default()));
    let mut clip = viewport;
    let mut current = node;
    for _ in 0..tree.len() {
        let Some(parent) = tree.visual_parent_of(current) else {
            break;
        };
        current = parent;
        let axes = OverflowAxes::of(styles.get(&parent));
        if !axes.clips() {
            continue;
        }
        let Some(bounds) = boxes.get(&parent).map(|layout| {
            crate::orchestration::overflow_padding_box(styles.get(&parent), layout.bounds)
        }) else {
            continue;
        };
        let (mut left, mut top, mut right, mut bottom) =
            (clip.origin.x, clip.origin.y, clip.right(), clip.bottom());
        if axes.inline_axis() != OVERFLOW_VISIBLE {
            left = left.max(bounds.origin.x);
            right = right.min(bounds.right());
        }
        if axes.block_axis() != OVERFLOW_VISIBLE {
            top = top.max(bounds.origin.y);
            bottom = bottom.min(bounds.bottom());
        }
        clip = Rect::new(
            Point::new(left, top),
            Size::new((right - left).max(0.0), (bottom - top).max(0.0)),
        );
    }
    (target, clip)
}
