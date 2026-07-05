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
///
/// [`Relative`](Self::Relative), [`Absolute`](Self::Absolute), and
/// [`Fixed`](Self::Fixed) each *establish a containing block* for their
/// positioned descendants ([`is_positioned`](Self::is_positioned)); only
/// [`Absolute`](Self::Absolute) and [`Fixed`](Self::Fixed) are *out of flow*
/// ([`is_out_of_flow`](Self::is_out_of_flow)).
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
    }

    #[test]
    fn only_absolute_and_fixed_are_out_of_flow() {
        assert!(Position::Absolute.is_out_of_flow());
        assert!(Position::Fixed.is_out_of_flow());
        assert!(!Position::Static.is_out_of_flow());
        assert!(!Position::Relative.is_out_of_flow());
    }

    #[test]
    fn every_scheme_but_static_is_positioned() {
        assert!(!Position::Static.is_positioned());
        assert!(Position::Relative.is_positioned());
        assert!(Position::Absolute.is_positioned());
        assert!(Position::Fixed.is_positioned());
    }
}
