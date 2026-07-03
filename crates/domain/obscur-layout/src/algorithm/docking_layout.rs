//! DockingLayout: dock children to the container edges (DockPanel style).

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{LayoutBox, LayoutStyle};

use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput};

/// The edge a docked child attaches to (spec §4.1).
///
/// [`Fill`](DockSide::Fill) is the default when a child has no configured side:
/// it consumes whatever remaining space is left, which conventionally makes the
/// last child expand to fill the container's centre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSide {
    /// Dock against the top edge, spanning the remaining width.
    Top,
    /// Dock against the bottom edge, spanning the remaining width.
    Bottom,
    /// Dock against the left edge, spanning the remaining height.
    Left,
    /// Dock against the right edge, spanning the remaining height.
    Right,
    /// Fill the entire remaining space.
    Fill,
}

/// Dock-to-edge layout, in the style of a WPF `DockPanel` (spec §4.1, §4.2).
///
/// Children are docked, in order, to the edges of a shrinking rectangle. Each
/// child claims a band along its assigned edge; that band is removed from the
/// remaining space, and the next child docks against the smaller rectangle that
/// is left. A [`Fill`](DockSide::Fill) child takes all remaining space.
///
/// # Configuration
///
/// This is a **self-configured** algorithm (the model has no dock-side field):
/// `sides` is parallel to the children by index. If a child has no corresponding
/// entry, it defaults to [`Fill`](DockSide::Fill); the last child conventionally
/// fills the centre when its side is `Fill`.
///
/// # Algorithm
///
/// A `remaining` rectangle starts at the padded content area. For each child in
/// order, looking up its side (defaulting to `Fill`):
///
/// * **Top** / **Bottom** — measured with its width fixed to the remaining width
///   and its natural height (up to the remaining height); arranged flush against
///   that edge; `remaining` shrinks vertically by the band height.
/// * **Left** / **Right** — measured with its height fixed to the remaining
///   height and its natural width (up to the remaining width); arranged flush
///   against that edge; `remaining` shrinks horizontally by the band width.
/// * **Fill** — measured under the whole `remaining` rectangle and arranged to
///   cover it; `remaining` collapses to zero, so any later children get no
///   space.
///
/// The node size is the bounding box of the docked children plus the container
/// `padding`, clamped into `constraints`.
///
/// # Out of scope
///
/// `LastChildFill` toggling, per-child dock spacing, and right-to-left edge
/// remapping are not modelled; docking is strictly in child order against the
/// four physical edges with a trailing fill.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DockingLayout {
    /// The dock side per child index; missing entries default to `Fill`.
    sides: Vec<DockSide>,
}

impl DockingLayout {
    /// Creates a docking layout with no configured sides (every child fills).
    #[must_use]
    pub fn new() -> Self {
        Self { sides: Vec::new() }
    }

    /// Returns a copy whose per-child dock sides are replaced by `sides`.
    #[must_use]
    pub fn with_sides(mut self, sides: Vec<DockSide>) -> Self {
        self.sides = sides;
        self
    }

    /// Appends `side` for the next child index.
    pub fn push_side(&mut self, side: DockSide) {
        self.sides.push(side);
    }

    /// The configured dock sides, parallel to the children by index.
    #[must_use]
    pub fn sides(&self) -> &[DockSide] {
        &self.sides
    }
}

impl LayoutAlgorithm for DockingLayout {
    fn layout(
        &self,
        style: &LayoutStyle,
        children: &[LayoutChild],
        constraints: BoxConstraints,
        measurer: &mut dyn ChildMeasurer,
    ) -> LayoutOutput {
        let padding = style.padding;
        let pad_x = padding.left + padding.right;
        let pad_y = padding.top + padding.bottom;

        let content_width = (constraints.max.width - pad_x).max(0.0);
        let content_height = (constraints.max.height - pad_y).max(0.0);

        // The space still available to dock into, shrinking as bands are claimed.
        let mut remaining = Rect::new(
            Point::new(padding.left, padding.top),
            Size::new(content_width, content_height),
        );

        let mut child_boxes = Vec::with_capacity(children.len());
        // Content-relative bounding box; start at the padding origin so an empty
        // container sizes to its padding only.
        let mut max_right = padding.left;
        let mut max_bottom = padding.top;

        for (index, _child) in children.iter().enumerate() {
            let side = self.sides.get(index).copied().unwrap_or(DockSide::Fill);
            let (origin, size) = match side {
                DockSide::Top => {
                    let cc = BoxConstraints {
                        min: Size::new(remaining.size.width, 0.0),
                        max: Size::new(remaining.size.width, remaining.size.height),
                    };
                    let measured = measurer.measure_child(index, cc);
                    let band = measured.height.min(remaining.size.height);
                    let arranged = (
                        remaining.origin,
                        Size::new(remaining.size.width, band),
                    );
                    remaining.origin.y += band;
                    remaining.size.height = (remaining.size.height - band).max(0.0);
                    arranged
                }
                DockSide::Bottom => {
                    let cc = BoxConstraints {
                        min: Size::new(remaining.size.width, 0.0),
                        max: Size::new(remaining.size.width, remaining.size.height),
                    };
                    let measured = measurer.measure_child(index, cc);
                    let band = measured.height.min(remaining.size.height);
                    let arranged = (
                        Point::new(remaining.origin.x, remaining.bottom() - band),
                        Size::new(remaining.size.width, band),
                    );
                    remaining.size.height = (remaining.size.height - band).max(0.0);
                    arranged
                }
                DockSide::Left => {
                    let cc = BoxConstraints {
                        min: Size::new(0.0, remaining.size.height),
                        max: Size::new(remaining.size.width, remaining.size.height),
                    };
                    let measured = measurer.measure_child(index, cc);
                    let band = measured.width.min(remaining.size.width);
                    let arranged = (
                        remaining.origin,
                        Size::new(band, remaining.size.height),
                    );
                    remaining.origin.x += band;
                    remaining.size.width = (remaining.size.width - band).max(0.0);
                    arranged
                }
                DockSide::Right => {
                    let cc = BoxConstraints {
                        min: Size::new(0.0, remaining.size.height),
                        max: Size::new(remaining.size.width, remaining.size.height),
                    };
                    let measured = measurer.measure_child(index, cc);
                    let band = measured.width.min(remaining.size.width);
                    let arranged = (
                        Point::new(remaining.right() - band, remaining.origin.y),
                        Size::new(band, remaining.size.height),
                    );
                    remaining.size.width = (remaining.size.width - band).max(0.0);
                    arranged
                }
                DockSide::Fill => {
                    let cc = BoxConstraints::loose(remaining.size);
                    let _ = measurer.measure_child(index, cc);
                    let arranged = (remaining.origin, remaining.size);
                    remaining.size = Size::ZERO;
                    arranged
                }
            };
            child_boxes.push(LayoutBox::new(Rect::new(origin, size)));
            max_right = max_right.max(origin.x + size.width);
            max_bottom = max_bottom.max(origin.y + size.height);
        }

        let border_box = Size::new(max_right + padding.right, max_bottom + padding.bottom);
        let size = constraints.constrain(border_box);

        LayoutOutput::new(size, child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::{DockSide, DockingLayout};
    use crate::algorithm::{FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::LayoutStyle;
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    fn child(raw: u64) -> LayoutChild {
        LayoutChild::new(NodeId::from_raw(raw), LayoutStyle::default())
    }

    #[test]
    fn top_then_fill_docks_band_and_remainder() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        // Top child measures 40 tall; fill child gets whatever it's handed.
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 40.0), Size::new(0.0, 0.0)]);
        let dock = DockingLayout::new().with_sides(vec![DockSide::Top, DockSide::Fill]);
        let out = dock.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Top band: full width, 40 tall, at the origin.
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size(), Size::new(200.0, 40.0));
        // Fill: the remaining rect below the top band (200 x 60 at y = 40).
        assert_eq!(b[1].origin(), Point::new(0.0, 40.0));
        assert_eq!(b[1].size(), Size::new(200.0, 60.0));
        // Container fills its constraints.
        assert_eq!(out.size(), Size::new(200.0, 100.0));
    }

    #[test]
    fn left_then_fill_docks_column_and_remainder() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(30.0, 0.0), Size::new(0.0, 0.0)]);
        let dock = DockingLayout::new().with_sides(vec![DockSide::Left, DockSide::Fill]);
        let out = dock.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Left band: 30 wide, full height.
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size(), Size::new(30.0, 100.0));
        // Fill: the remaining rect to the right (170 x 100 at x = 30).
        assert_eq!(b[1].origin(), Point::new(30.0, 0.0));
        assert_eq!(b[1].size(), Size::new(170.0, 100.0));
    }

    #[test]
    fn bottom_and_right_dock_against_far_edges() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 20.0), Size::new(25.0, 0.0)]);
        let dock = DockingLayout::new().with_sides(vec![DockSide::Bottom, DockSide::Right]);
        let out = dock.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Bottom band: 20 tall, flush with the bottom edge (y = 100 - 20 = 80).
        assert_eq!(b[0].origin(), Point::new(0.0, 80.0));
        assert_eq!(b[0].size(), Size::new(200.0, 20.0));
        // Right band: 25 wide, full remaining height (80), flush with the right
        // edge (x = 200 - 25 = 175).
        assert_eq!(b[1].origin(), Point::new(175.0, 0.0));
        assert_eq!(b[1].size(), Size::new(25.0, 80.0));
    }

    #[test]
    fn push_side_appends_configuration() {
        let mut dock = DockingLayout::new();
        dock.push_side(DockSide::Left);
        dock.push_side(DockSide::Fill);
        assert_eq!(dock.sides(), &[DockSide::Left, DockSide::Fill]);
    }

    #[test]
    fn empty_container_sizes_to_padding() {
        let style = LayoutStyle {
            padding: Insets::uniform(4.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::default();
        let out = DockingLayout::default().layout(
            &style,
            &[],
            BoxConstraints::loose(Size::new(50.0, 50.0)),
            &mut measurer,
        );
        assert!(out.is_empty());
        assert_eq!(out.size(), Size::new(8.0, 8.0));
    }
}
