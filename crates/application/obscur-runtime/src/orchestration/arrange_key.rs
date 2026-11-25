use obscur_geometry::{BoxConstraints, Point, Rect};

#[doc = include_str!("../../docs/api/orchestration/arrange_cache.doc/ArrangeKey.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct ArrangeKey {
    /// The absolute origin the box was written at.
    origin: (u32, u32),
    #[doc = include_str!("../../docs/api/orchestration/arrange_cache.doc/constraints.md")]
    constraints: (u32, u32, u32, u32),
    /// The containing block rect: `x`, `y`, `width`, `height`.
    containing_block: (u32, u32, u32, u32),
    /// The viewport rect: `x`, `y`, `width`, `height`.
    viewport: (u32, u32, u32, u32),
    /// The viewport scroll offset.
    scroll: (u32, u32),
    natural_block_size: bool,
    percent_basis: Option<(u32, u32)>,
}

impl ArrangeKey {
    /// The key for one [`arrange`](super::arrange_pass::arrange) call.
    pub(super) fn new(
        origin: Point,
        constraints: BoxConstraints,
        containing_block: Rect,
        viewport: Rect,
        scroll: (f32, f32),
    ) -> Self {
        let rect = |r: Rect| {
            (
                r.origin.x.to_bits(),
                r.origin.y.to_bits(),
                r.size.width.to_bits(),
                r.size.height.to_bits(),
            )
        };
        Self {
            origin: (origin.x.to_bits(), origin.y.to_bits()),
            constraints: (
                constraints.min.width.to_bits(),
                constraints.min.height.to_bits(),
                constraints.max.width.to_bits(),
                constraints.max.height.to_bits(),
            ),
            containing_block: rect(containing_block),
            viewport: rect(viewport),
            scroll: (scroll.0.to_bits(), scroll.1.to_bits()),
            natural_block_size: false,
            percent_basis: None,
        }
    }
    /// Includes the containing size when the box uses percentage padding.
    pub(super) fn with_padding_basis(mut self, constraints: BoxConstraints) -> Self {
        self.percent_basis = Some((
            constraints.percent_width().to_bits(),
            constraints.percent_height().to_bits(),
        ));
        self
    }

    /// Records whether the parent assigned a size derived from content.
    pub(super) const fn with_natural_block_size(mut self, natural: bool) -> Self {
        self.natural_block_size = natural;
        self
    }
}
