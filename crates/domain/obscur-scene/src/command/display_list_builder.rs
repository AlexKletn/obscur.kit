//! An incremental builder for [`DisplayList`]s.

use obscur_geometry::{CornerRadii, Point, Rect};

use crate::clip::Clip;
use crate::color::Color;
use crate::layer::SceneLayer;
use crate::paint::{Brush, Paint, Stroke};
use crate::path::Path;
use crate::text::GlyphRun;

use super::{DisplayList, PaintCommand};

/// A builder that appends [`PaintCommand`]s in paint order while tracking
/// push/pop nesting (spec §5.1).
///
/// Commands are appended in the exact order they are issued; that order becomes
/// the display list's deterministic paint order. The builder maintains a live
/// nesting [`depth`] using saturating arithmetic, so a stray pop can never
/// underflow.
///
/// The builder does **not** enforce balance: [`build`] returns the accumulated
/// commands verbatim without auto-closing any open clips or layers. An
/// unbalanced builder therefore yields an unbalanced [`DisplayList`], which the
/// caller can detect via [`DisplayList::is_balanced`]. A `pop_*` call on an
/// empty stack is recorded verbatim as its command, but keeps [`depth`] clamped
/// at zero so it never underflows or panics.
///
/// [`depth`]: DisplayListBuilder::depth
/// [`build`]: DisplayListBuilder::build
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DisplayListBuilder {
    commands: Vec<PaintCommand>,
    depth: u32,
}

impl DisplayListBuilder {
    /// Creates an empty builder with a nesting depth of zero.
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            depth: 0,
        }
    }

    /// Appends a [`PaintCommand::DrawPath`] painting `path` with `paint`.
    /// Returns `&mut Self` for chaining.
    pub fn draw(&mut self, path: Path, paint: Paint) -> &mut Self {
        self.commands.push(PaintCommand::DrawPath { path, paint });
        self
    }

    /// Appends a fill: a [`PaintCommand::DrawPath`] with `Paint::fill(brush)`.
    /// Returns `&mut Self` for chaining.
    pub fn fill(&mut self, path: Path, brush: Brush) -> &mut Self {
        self.draw(path, Paint::fill(brush))
    }

    /// Appends a filled rectangle, building the path with [`Path::rect`] and
    /// filling it with `brush`. Returns `&mut Self` for chaining.
    pub fn fill_rect(&mut self, rect: Rect, brush: Brush) -> &mut Self {
        self.fill(Path::rect(rect), brush)
    }

    /// Appends a [`PaintCommand::DrawText`] drawing the pre-shaped `run` at
    /// `origin` (its top-left in absolute scene coordinates), painted with
    /// `brush`. This is a leaf command; the run's glyphs were shaped once in the
    /// `obscur-text` domain. Returns `&mut Self` for chaining.
    pub fn draw_text(&mut self, origin: Point, run: GlyphRun, brush: Brush) -> &mut Self {
        self.commands.push(PaintCommand::DrawText { origin, run, brush });
        self
    }

    /// Appends a [`PaintCommand::DrawShadow`] painting a CSS `box-shadow` for the
    /// border box `rect` with corner `radii`, the resolved `color`, blur radius
    /// `blur`, `spread`, `offset`, and the `inset` flag. This is a leaf command
    /// (no nesting effect). Returns `&mut Self` for chaining.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_shadow(
        &mut self,
        rect: Rect,
        radii: CornerRadii,
        color: Color,
        blur: f32,
        spread: f32,
        offset: (f32, f32),
        inset: bool,
    ) -> &mut Self {
        self.commands.push(PaintCommand::DrawShadow {
            rect,
            radii,
            color,
            blur,
            spread,
            offset,
            inset,
        });
        self
    }

    /// Appends a stroke: a [`PaintCommand::DrawPath`] with
    /// `Paint::stroke(stroke)`. Returns `&mut Self` for chaining.
    pub fn stroke(&mut self, path: Path, stroke: Stroke) -> &mut Self {
        self.draw(path, Paint::stroke(stroke))
    }

    /// Appends a [`PaintCommand::PushClip`] and increments the nesting depth.
    /// Returns `&mut Self` for chaining.
    pub fn push_clip(&mut self, clip: Clip) -> &mut Self {
        self.commands.push(PaintCommand::PushClip(clip));
        self.depth = self.depth.saturating_add(1);
        self
    }

    /// Appends a [`PaintCommand::PopClip`] and decrements the nesting depth
    /// (saturating at zero). The command is always recorded, even if the stack
    /// is already empty. Returns `&mut Self` for chaining.
    pub fn pop_clip(&mut self) -> &mut Self {
        self.commands.push(PaintCommand::PopClip);
        self.depth = self.depth.saturating_sub(1);
        self
    }

    /// Appends a [`PaintCommand::PushLayer`] and increments the nesting depth.
    /// Returns `&mut Self` for chaining.
    pub fn push_layer(&mut self, layer: SceneLayer) -> &mut Self {
        self.commands.push(PaintCommand::PushLayer(layer));
        self.depth = self.depth.saturating_add(1);
        self
    }

    /// Appends a [`PaintCommand::PopLayer`] and decrements the nesting depth
    /// (saturating at zero). The command is always recorded, even if the stack
    /// is already empty. Returns `&mut Self` for chaining.
    pub fn pop_layer(&mut self) -> &mut Self {
        self.commands.push(PaintCommand::PopLayer);
        self.depth = self.depth.saturating_sub(1);
        self
    }

    /// The current nesting depth: the number of clips/layers pushed but not yet
    /// popped. Clamped at zero, so a stray pop never underflows.
    #[must_use]
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// Whether the builder is currently balanced, i.e. [`depth`] is zero.
    ///
    /// Note this tracks the running depth clamped at zero; a builder with a
    /// stray pop followed by a balancing push reports balanced here, yet the
    /// resulting list is not — use [`DisplayList::is_balanced`] on the built
    /// list for the authoritative check.
    ///
    /// [`depth`]: DisplayListBuilder::depth
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        self.depth == 0
    }

    /// Consumes the builder and returns the accumulated [`DisplayList`] **as
    /// is**, in paint order.
    ///
    /// This does **not** auto-close open clips or layers: if the builder is
    /// unbalanced, the returned list is unbalanced too. Callers who need the
    /// guarantee should verify [`DisplayList::is_balanced`].
    #[must_use]
    pub fn build(self) -> DisplayList {
        DisplayList::from_commands(self.commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use obscur_geometry::{Point, Size};

    fn rect() -> Rect {
        Rect::new(Point::new(0.0, 0.0), Size::new(4.0, 4.0))
    }

    #[test]
    fn balanced_sequence_preserves_order_and_returns_to_zero() {
        let mut builder = DisplayListBuilder::new();
        builder
            .push_layer(SceneLayer::new())
            .push_clip(Clip::rect(rect()))
            .fill_rect(rect(), Brush::solid(Color::WHITE))
            .pop_clip()
            .pop_layer();

        assert_eq!(builder.depth(), 0);
        assert!(builder.is_balanced());

        let list = builder.build();
        let commands = list.commands();
        assert_eq!(commands.len(), 5);
        assert!(matches!(commands[0], PaintCommand::PushLayer(_)));
        assert!(matches!(commands[1], PaintCommand::PushClip(_)));
        assert!(matches!(commands[2], PaintCommand::DrawPath { .. }));
        assert_eq!(commands[3], PaintCommand::PopClip);
        assert_eq!(commands[4], PaintCommand::PopLayer);
        assert!(list.is_balanced());
    }

    #[test]
    fn missing_pop_builds_unbalanced_list() {
        let mut builder = DisplayListBuilder::new();
        builder
            .push_clip(Clip::rect(rect()))
            .fill_rect(rect(), Brush::solid(Color::BLACK));

        assert_eq!(builder.depth(), 1);
        assert!(!builder.is_balanced());

        let list = builder.build();
        assert!(!list.is_balanced());
    }

    #[test]
    fn stray_pop_on_empty_builder_stays_at_zero() {
        let mut builder = DisplayListBuilder::new();
        builder.pop_clip();

        // Depth is clamped: no underflow.
        assert_eq!(builder.depth(), 0);
        // The command is still recorded verbatim.
        let list = builder.build();
        assert_eq!(list.len(), 1);
        assert_eq!(list.commands()[0], PaintCommand::PopClip);
        // A stray pop makes the list unbalanced.
        assert!(!list.is_balanced());
    }

    #[test]
    fn fill_and_stroke_produce_expected_paints() {
        let mut builder = DisplayListBuilder::new();
        builder
            .fill(Path::rect(rect()), Brush::solid(Color::WHITE))
            .stroke(
                Path::rect(rect()),
                Stroke::new(Brush::solid(Color::BLACK), 2.0),
            )
            .fill_rect(rect(), Brush::solid(Color::WHITE));

        let list = builder.build();
        match &list.commands()[0] {
            PaintCommand::DrawPath { paint, .. } => assert!(paint.is_fill()),
            other => panic!("expected DrawPath, got {other:?}"),
        }
        match &list.commands()[1] {
            PaintCommand::DrawPath { paint, .. } => assert!(paint.is_stroke()),
            other => panic!("expected DrawPath, got {other:?}"),
        }
        match &list.commands()[2] {
            PaintCommand::DrawPath { path, paint } => {
                assert!(paint.is_fill());
                assert_eq!(path, &Path::rect(rect()));
            }
            other => panic!("expected DrawPath, got {other:?}"),
        }
    }

    #[test]
    fn draw_text_appends_a_leaf_text_command() {
        use crate::text::PositionedGlyph;
        let run = GlyphRun::new(
            vec![PositionedGlyph::new(u32::from('h'), 0, 0.0, 18.0, 10.8)],
            18.0,
            "hello",
        );
        let mut builder = DisplayListBuilder::new();
        builder.draw_text(Point::new(3.0, 7.0), run.clone(), Brush::solid(Color::WHITE));

        assert!(builder.is_balanced());
        let list = builder.build();
        assert_eq!(list.len(), 1);
        assert_eq!(
            list.commands()[0],
            PaintCommand::DrawText {
                origin: Point::new(3.0, 7.0),
                run,
                brush: Brush::solid(Color::WHITE),
            }
        );
        // A list containing DrawText is still balanced (it is a leaf).
        assert!(list.is_balanced());
    }

    #[test]
    fn draw_shadow_appends_a_balanced_leaf_shadow() {
        let mut builder = DisplayListBuilder::new();
        builder.draw_shadow(
            rect(),
            CornerRadii::uniform(3.0),
            Color::BLACK,
            6.0,
            1.0,
            (2.0, 4.0),
            false,
        );
        assert!(builder.is_balanced());
        let list = builder.build();
        assert_eq!(list.len(), 1);
        assert_eq!(
            list.commands()[0],
            PaintCommand::DrawShadow {
                rect: rect(),
                radii: CornerRadii::uniform(3.0),
                color: Color::BLACK,
                blur: 6.0,
                spread: 1.0,
                offset: (2.0, 4.0),
                inset: false,
            }
        );
        // A DrawShadow is a leaf, so the list stays balanced.
        assert!(list.is_balanced());
    }
}
