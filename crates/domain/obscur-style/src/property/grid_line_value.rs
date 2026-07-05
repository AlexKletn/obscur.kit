//! `GridLineValue`: one style-local edge of a grid item's line placement.

/// One placement edge of a grid item along a single axis, in style vocabulary.
///
/// This is the `obscur-style` twin of `obscur_layout`'s `GridLine`. `obscur-style`
/// is a pure domain crate that does not depend on `obscur-layout`, so an authored
/// `grid-column` / `grid-row` edge is modelled here and the runtime converts each
/// [`GridLineValue`] to the layout crate's own `GridLine` when it bridges a
/// computed style into a layout input (spec §4.13).
///
/// A CSS `grid-column` / `grid-row` is a `start / end` pair; each side is one
/// [`GridLineValue`]:
///
/// * [`Auto`](GridLineValue::Auto) — no explicit line; the item is auto-placed on
///   this edge (the initial value).
/// * [`Line`](GridLineValue::Line) — a **1-based** grid line index (`grid-column: 2`
///   is line `2`). A negative index counts back from the end of the explicit grid
///   (`-1` is the last line); the layout crate resolves it against the real track
///   count. Named lines are out of scope.
/// * [`Span`](GridLineValue::Span) — `span N`: the edge is placed `N` tracks away
///   from the opposite edge.
/// * [`Named`](GridLineValue::Named) — a `<custom-ident>` line name
///   (`grid-column: sidebar-start`). The name cannot be resolved on the item
///   alone: it refers to a line named on the item's **container**'s
///   `grid-template-columns` / `grid-template-rows` (or an implicit
///   `<area>-start` / `<area>-end` line), so the runtime resolves it against the
///   parent's computed line-name map, exactly as `grid-area: <name>` is resolved
///   against the parent's `grid-template-areas` (spec §4.13).
///
/// # Determinism
///
/// `GridLineValue` has no interior mutability, so placement resolves
/// deterministically from the item styles (and, for [`Named`](GridLineValue::Named),
/// the parent's computed template) alone. The [`Named`](GridLineValue::Named)
/// variant carries an owned name, so this is [`Clone`] but not [`Copy`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridLineValue {
    /// No explicit line on this edge; the item is auto-placed here.
    Auto,
    /// A 1-based grid line index.
    Line(i32),
    /// `span N`: this edge is `N` tracks from the opposite edge.
    Span(u16),
    /// A `<custom-ident>` line name, resolved against the container's named lines
    /// in the runtime (spec §4.13).
    Named(Box<str>),
}

impl Default for GridLineValue {
    /// The initial value: [`Auto`](GridLineValue::Auto) (pure auto-placement).
    fn default() -> Self {
        GridLineValue::Auto
    }
}

#[cfg(test)]
mod tests {
    use super::GridLineValue;

    #[test]
    fn default_is_auto() {
        assert_eq!(GridLineValue::default(), GridLineValue::Auto);
    }

    #[test]
    fn variants_carry_their_payloads() {
        assert_eq!(GridLineValue::Line(3), GridLineValue::Line(3));
        assert_eq!(GridLineValue::Span(2), GridLineValue::Span(2));
        assert_ne!(GridLineValue::Line(1), GridLineValue::Span(1));
        assert_eq!(
            GridLineValue::Named("main".into()),
            GridLineValue::Named("main".into())
        );
        assert_ne!(
            GridLineValue::Named("main".into()),
            GridLineValue::Named("side".into())
        );
    }
}
