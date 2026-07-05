//! A layout node's positioning scheme (`position`).

/// How a node is positioned relative to the normal flow (spec §4.1; CSS
/// `position`).
///
/// All four CSS positioning schemes are modelled so the vocabulary is stable,
/// but only [`Static`](Self::Static) and [`Relative`](Self::Relative) affect
/// layout in this step:
///
/// * [`Static`](Self::Static) — the default. The node is placed by its
///   formatting context and its insets are ignored.
/// * [`Relative`](Self::Relative) — the node stays **in flow** (its flow box and
///   its siblings are unchanged), then its final position (and its whole
///   subtree) is offset by its resolved insets.
/// * [`Absolute`](Self::Absolute) / [`Fixed`](Self::Fixed) — out-of-flow
///   schemes. They are defined here as placeholders but are **treated as
///   [`Static`](Self::Static) for layout in this step** (real out-of-flow
///   positioning is a later step). [`is_relative`](Self::is_relative) therefore
///   returns `false` for them, so they contribute no offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    /// The node is laid out in normal flow; insets are ignored (CSS `static`).
    #[default]
    Static,
    /// The node is laid out in normal flow, then offset by its insets without
    /// affecting its siblings (CSS `relative`).
    Relative,
    /// Out-of-flow, positioned against the nearest positioned ancestor (CSS
    /// `absolute`). Treated as [`Static`](Self::Static) for layout this step.
    Absolute,
    /// Out-of-flow, positioned against the viewport (CSS `fixed`). Treated as
    /// [`Static`](Self::Static) for layout this step.
    Fixed,
}

impl Position {
    /// Returns `true` only for [`Relative`](Self::Relative) — the one scheme that
    /// offsets an in-flow node by its insets in this step.
    ///
    /// [`Absolute`](Self::Absolute) and [`Fixed`](Self::Fixed) return `false`
    /// because they are treated as [`Static`](Self::Static) until out-of-flow
    /// positioning lands.
    #[must_use]
    pub const fn is_relative(self) -> bool {
        matches!(self, Self::Relative)
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
        // Absolute/Fixed are placeholders treated as static this step.
        assert!(!Position::Absolute.is_relative());
        assert!(!Position::Fixed.is_relative());
    }
}
