//! The effect a drop will have on the dragged data (spec §6.21 drag and drop —
//! effects).

/// The effect a completed drop will have on the dragged data (spec §6.21
/// effects).
///
/// A drag negotiates an effect between what the source permits and what the
/// target allows: dropping may [`Copy`](DragEffect::Copy) the data (leaving the
/// original), [`Move`](DragEffect::Move) it (consuming the original), or create
/// a [`Link`](DragEffect::Link) / reference to it. [`None`](DragEffect::None) is
/// the absence of any acceptable effect — a drop under `None` is a no-op that
/// cancels rather than commits. Use
/// [`first_allowed`](DragEffect::first_allowed) to negotiate the effect a drop
/// should carry from a target's allowed set and a preferred choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DragEffect {
    /// No acceptable effect — a drop is a no-op / cancel (spec §6.21).
    #[default]
    None,
    /// Copy the data, leaving the original in place (spec §6.21).
    Copy,
    /// Move the data, consuming the original (spec §6.21).
    Move,
    /// Create a link / reference to the data (spec §6.21).
    Link,
}

impl DragEffect {
    /// Returns whether this is [`None`](DragEffect::None) — no acceptable effect
    /// (spec §6.21).
    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns whether this is [`Move`](DragEffect::Move) (spec §6.21).
    #[must_use]
    pub const fn is_move(self) -> bool {
        matches!(self, Self::Move)
    }

    /// Negotiates the effect a drop should carry (spec §6.21 effect
    /// negotiation).
    ///
    /// Returns `preferred` when it is present in `allowed` and is not
    /// [`None`](DragEffect::None); otherwise the first non-`None` entry of
    /// `allowed`; otherwise [`None`](DragEffect::None) when `allowed` offers no
    /// acceptable effect.
    #[must_use]
    pub fn first_allowed(allowed: &[DragEffect], preferred: DragEffect) -> DragEffect {
        if !preferred.is_none() && allowed.contains(&preferred) {
            return preferred;
        }
        for &effect in allowed {
            if !effect.is_none() {
                return effect;
            }
        }
        Self::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_none() {
        assert_eq!(DragEffect::default(), DragEffect::None);
        assert!(DragEffect::None.is_none());
        assert!(!DragEffect::Copy.is_none());
        assert!(DragEffect::Move.is_move());
        assert!(!DragEffect::Copy.is_move());
    }

    #[test]
    fn first_allowed_returns_preferred_when_present() {
        let allowed = [DragEffect::Copy, DragEffect::Move];
        assert_eq!(
            DragEffect::first_allowed(&allowed, DragEffect::Move),
            DragEffect::Move
        );
    }

    #[test]
    fn first_allowed_falls_through_when_preferred_none() {
        let allowed = [DragEffect::Copy, DragEffect::Move];
        assert_eq!(
            DragEffect::first_allowed(&allowed, DragEffect::None),
            DragEffect::Copy
        );
    }

    #[test]
    fn first_allowed_falls_through_when_preferred_absent() {
        let allowed = [DragEffect::Copy];
        assert_eq!(
            DragEffect::first_allowed(&allowed, DragEffect::Link),
            DragEffect::Copy
        );
    }

    #[test]
    fn first_allowed_empty_is_none() {
        assert_eq!(
            DragEffect::first_allowed(&[], DragEffect::Copy),
            DragEffect::None
        );
    }

    #[test]
    fn first_allowed_skips_none_entries() {
        let allowed = [DragEffect::None, DragEffect::Link];
        assert_eq!(
            DragEffect::first_allowed(&allowed, DragEffect::None),
            DragEffect::Link
        );
    }
}
