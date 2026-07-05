//! A layout node's positioning scheme (`position`).

/// How a node is positioned relative to the normal flow (spec §4.1; CSS
/// `position`).
///
/// All four CSS positioning schemes are modelled and honoured by the layout
/// coordinator:
///
/// * [`Static`](Self::Static) — the default. The node is placed by its
///   formatting context and its insets are ignored.
/// * [`Relative`](Self::Relative) — the node stays **in flow** (its flow box and
///   its siblings are unchanged), then its final position (and its whole
///   subtree) is offset by its resolved insets.
/// * [`Absolute`](Self::Absolute) — **out of flow**: the node does not
///   participate in its parent's formatting context (it neither affects nor is
///   affected by in-flow siblings) and is positioned by its insets within the
///   padding box of its nearest positioned ancestor (its *containing block*).
/// * [`Fixed`](Self::Fixed) — out of flow like [`Absolute`](Self::Absolute), but
///   its containing block is always the viewport, ignoring ancestors.
/// * [`Sticky`](Self::Sticky) — a **hybrid**. The node stays **in flow** exactly
///   like [`Relative`](Self::Relative) (its flow box and its siblings are
///   unchanged), but its *used* position is then clamped so that, once its scroll
///   container has scrolled past its inset threshold, the node sticks to the
///   scrollport edge — never moving above its static position and never past its
///   containing block. With a zero scroll offset (or before the threshold is
///   reached) its used position equals its static position, so it is transparent.
///   The single scroll container is currently the **viewport**; per-container
///   overflow scroll containers are a follow-up.
///
/// [`Relative`](Self::Relative), [`Absolute`](Self::Absolute),
/// [`Fixed`](Self::Fixed), and [`Sticky`](Self::Sticky) each *establish a
/// containing block* for their positioned descendants
/// ([`is_positioned`](Self::is_positioned)); only [`Absolute`](Self::Absolute) and
/// [`Fixed`](Self::Fixed) are *out of flow* ([`is_out_of_flow`](Self::is_out_of_flow)).
/// [`Sticky`](Self::Sticky), like [`Relative`](Self::Relative), is **in flow**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    /// The node is laid out in normal flow; insets are ignored (CSS `static`).
    #[default]
    Static,
    /// The node is laid out in normal flow, then offset by its insets without
    /// affecting its siblings (CSS `relative`).
    Relative,
    /// Out-of-flow, positioned against the nearest positioned ancestor's padding
    /// box (CSS `absolute`).
    Absolute,
    /// Out-of-flow, positioned against the viewport (CSS `fixed`).
    Fixed,
    /// In flow like [`Relative`](Self::Relative), but its used position is
    /// scroll-clamped to stick to the scrollport edge within its containing block
    /// (CSS `sticky`).
    Sticky,
}

impl Position {
    /// Returns `true` only for [`Relative`](Self::Relative) — the one scheme that
    /// offsets an **in-flow** node by its insets.
    ///
    /// [`Absolute`](Self::Absolute) and [`Fixed`](Self::Fixed) return `false`:
    /// they are out of flow and are positioned by the coordinator's out-of-flow
    /// path, not by the in-flow relative offset.
    #[must_use]
    pub const fn is_relative(self) -> bool {
        matches!(self, Self::Relative)
    }

    /// Returns `true` for the **out-of-flow** schemes
    /// ([`Absolute`](Self::Absolute) / [`Fixed`](Self::Fixed)).
    ///
    /// An out-of-flow node is excluded from its parent's flow/flex/grid
    /// algorithm entirely: it neither contributes to the parent's content size
    /// nor shifts its in-flow siblings, and is positioned separately against its
    /// containing block.
    #[must_use]
    pub const fn is_out_of_flow(self) -> bool {
        matches!(self, Self::Absolute | Self::Fixed)
    }

    /// Returns `true` only for [`Sticky`](Self::Sticky) — the one **in-flow**
    /// scheme whose used position is scroll-clamped to the scrollport edge.
    ///
    /// A sticky node is *not* [`is_relative`](Self::is_relative) (it is not
    /// offset by a fixed inset delta) and is *not*
    /// [`is_out_of_flow`](Self::is_out_of_flow) (it keeps its flow box and never
    /// perturbs its siblings); the coordinator recognises it here and applies the
    /// per-axis sticky clamp instead of the relative offset.
    #[must_use]
    pub const fn is_sticky(self) -> bool {
        matches!(self, Self::Sticky)
    }

    /// Returns `true` for any *positioned* scheme — everything except
    /// [`Static`](Self::Static).
    ///
    /// A positioned node **establishes a containing block**: its own padding box
    /// becomes the containing block against which its
    /// [`Absolute`](Self::Absolute) descendants resolve their insets.
    #[must_use]
    pub const fn is_positioned(self) -> bool {
        !matches!(self, Self::Static)
    }
}

#[cfg(test)]
mod tests {
    use super::Position;

    #[test]
    fn default_is_static() {
        assert_eq!(Position::default(), Position::Static);
    }

    #[test]
    fn only_relative_offsets_in_flow() {
        assert!(Position::Relative.is_relative());
        assert!(!Position::Static.is_relative());
        // Absolute/Fixed are out of flow, not offset in flow.
        assert!(!Position::Absolute.is_relative());
        assert!(!Position::Fixed.is_relative());
        // Sticky is in flow, but is NOT a plain relative offset.
        assert!(!Position::Sticky.is_relative());
    }

    #[test]
    fn only_absolute_and_fixed_are_out_of_flow() {
        assert!(Position::Absolute.is_out_of_flow());
        assert!(Position::Fixed.is_out_of_flow());
        assert!(!Position::Static.is_out_of_flow());
        assert!(!Position::Relative.is_out_of_flow());
        // Sticky stays IN flow (siblings unaffected), like relative.
        assert!(!Position::Sticky.is_out_of_flow());
    }

    #[test]
    fn only_sticky_is_sticky() {
        assert!(Position::Sticky.is_sticky());
        assert!(!Position::Static.is_sticky());
        assert!(!Position::Relative.is_sticky());
        assert!(!Position::Absolute.is_sticky());
        assert!(!Position::Fixed.is_sticky());
    }

    #[test]
    fn every_scheme_but_static_is_positioned() {
        assert!(!Position::Static.is_positioned());
        assert!(Position::Relative.is_positioned());
        assert!(Position::Absolute.is_positioned());
        assert!(Position::Fixed.is_positioned());
        // Sticky establishes a containing block for its descendants, like relative.
        assert!(Position::Sticky.is_positioned());
    }
}
