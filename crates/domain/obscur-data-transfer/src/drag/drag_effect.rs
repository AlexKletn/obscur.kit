#![doc = include_str!("drag_effect.doc/_module.md")]

#[doc = include_str!("drag_effect.doc/DragEffect.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DragEffect {
    #[doc = include_str!("drag_effect.doc/None.md")]
    #[default]
    None,
    #[doc = include_str!("drag_effect.doc/Copy.md")]
    Copy,
    #[doc = include_str!("drag_effect.doc/Move.md")]
    Move,
    #[doc = include_str!("drag_effect.doc/Link.md")]
    Link,
}

impl DragEffect {
    #[doc = include_str!("drag_effect.doc/is_none.md")]
    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    #[doc = include_str!("drag_effect.doc/is_move.md")]
    #[must_use]
    pub const fn is_move(self) -> bool {
        matches!(self, Self::Move)
    }

    #[doc = include_str!("drag_effect.doc/first_allowed.md")]
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
