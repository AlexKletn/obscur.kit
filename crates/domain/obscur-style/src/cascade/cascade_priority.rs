//! The total-order sort key that resolves a cascade conflict:
//! [`CascadePriority`] (§4.14).

use crate::cascade::{CascadeOrigin, ImportantFlag};
use crate::selector::SelectorSpecificity;
use core::cmp::Ordering;

#[doc = include_str!("../../docs/api/cascade/cascade_priority.doc/CascadePriority.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CascadePriority {
    /// The precedence band computed from `(origin, important)` — see
    /// [`CascadePriority::band`].
    band: u8,
    /// The selector specificity, compared within a band.
    specificity: SelectorSpecificity,
    /// The declaration's position within its origin stream; later wins.
    source_order: u32,
    /// The origin the priority was built from (retained for inspection).
    origin: CascadeOrigin,
    /// The importance flag the priority was built from (retained for
    /// inspection).
    important: ImportantFlag,
}

/// Band value for authored `!important` declarations, added to the origin rank.
const AUTHORED_IMPORTANT_BASE: u8 = 10;
/// Band value for the runtime-override layer when the declaration is normal.
const RUNTIME_NORMAL_BAND: u8 = 20;
/// Band value for the runtime-override layer when the declaration is important.
const RUNTIME_IMPORTANT_BAND: u8 = 21;

impl CascadePriority {
    #[doc = include_str!("../../docs/api/cascade/cascade_priority.doc/new.md")]
    #[must_use]
    pub const fn new(
        origin: CascadeOrigin,
        important: ImportantFlag,
        specificity: SelectorSpecificity,
        source_order: u32,
    ) -> Self {
        Self {
            band: Self::compute_band(origin, important),
            specificity,
            source_order,
            origin,
            important,
        }
    }

    #[doc = include_str!("../../docs/api/cascade/cascade_priority.doc/compute_band.md")]
    const fn compute_band(origin: CascadeOrigin, important: ImportantFlag) -> u8 {
        if origin.is_runtime_override() {
            match important {
                ImportantFlag::Normal => RUNTIME_NORMAL_BAND,
                ImportantFlag::Important => RUNTIME_IMPORTANT_BAND,
            }
        } else {
            // `origin` is one of the five authored origins here, whose
            // discriminants are 0..=4 in ascending precedence order.
            let origin_rank = origin as u8;
            match important {
                ImportantFlag::Normal => origin_rank,
                ImportantFlag::Important => AUTHORED_IMPORTANT_BASE + origin_rank,
            }
        }
    }

    #[doc = include_str!("../../docs/api/cascade/cascade_priority.doc/band_2.md")]
    #[must_use]
    pub const fn band(self) -> u8 {
        self.band
    }

    /// Returns the origin this priority was built from.
    #[must_use]
    pub const fn origin(self) -> CascadeOrigin {
        self.origin
    }

    /// Returns the importance flag this priority was built from.
    #[must_use]
    pub const fn important(self) -> ImportantFlag {
        self.important
    }

    /// Returns the selector specificity this priority was built from.
    #[must_use]
    pub const fn specificity(self) -> SelectorSpecificity {
        self.specificity
    }

    /// Returns the source order this priority was built from.
    #[must_use]
    pub const fn source_order(self) -> u32 {
        self.source_order
    }

    /// The ordering key: `(band, specificity, source_order)`, most significant
    /// first.
    const fn key(self) -> (u8, SelectorSpecificity, u32) {
        (self.band, self.specificity, self.source_order)
    }
}

impl PartialOrd for CascadePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CascadePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(a: u16, b: u16, c: u16) -> SelectorSpecificity {
        SelectorSpecificity::new(a, b, c)
    }

    #[test]
    fn authored_important_beats_authored_normal_of_higher_origin() {
        // An `!important` Theme declaration must beat a *normal* Inline one,
        // even though Inline is a higher origin than Theme (§4.14).
        let important_theme = CascadePriority::new(
            CascadeOrigin::Theme,
            ImportantFlag::Important,
            spec(9, 9, 9),
            0,
        );
        let normal_inline = CascadePriority::new(
            CascadeOrigin::Inline,
            ImportantFlag::Normal,
            spec(0, 0, 0),
            9999,
        );
        assert!(important_theme > normal_inline);
    }

    #[test]
    fn runtime_override_beats_authored_important() {
        // The runtime-override layer sits above all authored CSS, including
        // authored `!important` (§4.14). Even a *normal* runtime override with
        // trivial specificity beats an authored important Inline declaration.
        let runtime_normal = CascadePriority::new(
            CascadeOrigin::RuntimeOverride,
            ImportantFlag::Normal,
            spec(0, 0, 0),
            0,
        );
        let authored_important_inline = CascadePriority::new(
            CascadeOrigin::Inline,
            ImportantFlag::Important,
            spec(9, 9, 9),
            9999,
        );
        assert!(runtime_normal > authored_important_inline);
    }

    #[test]
    fn runtime_important_beats_runtime_normal() {
        let runtime_important = CascadePriority::new(
            CascadeOrigin::RuntimeOverride,
            ImportantFlag::Important,
            spec(0, 0, 0),
            0,
        );
        let runtime_normal = CascadePriority::new(
            CascadeOrigin::RuntimeOverride,
            ImportantFlag::Normal,
            spec(9, 9, 9),
            9999,
        );
        assert!(runtime_important > runtime_normal);
    }

    #[test]
    fn within_band_higher_specificity_wins() {
        let low = CascadePriority::new(
            CascadeOrigin::GlobalApp,
            ImportantFlag::Normal,
            spec(0, 1, 0),
            100,
        );
        let high = CascadePriority::new(
            CascadeOrigin::GlobalApp,
            ImportantFlag::Normal,
            spec(1, 0, 0),
            0,
        );
        // Higher specificity wins even though `low` has a later source order.
        assert!(high > low);
    }

    #[test]
    fn within_band_equal_specificity_later_source_order_wins() {
        let earlier = CascadePriority::new(
            CascadeOrigin::GlobalApp,
            ImportantFlag::Normal,
            spec(1, 1, 1),
            5,
        );
        let later = CascadePriority::new(
            CascadeOrigin::GlobalApp,
            ImportantFlag::Normal,
            spec(1, 1, 1),
            6,
        );
        assert!(later > earlier);
    }

    #[test]
    fn normal_origin_order_matches_spec() {
        let system = CascadePriority::new(
            CascadeOrigin::SystemDefault,
            ImportantFlag::Normal,
            spec(9, 9, 9),
            0,
        );
        let theme = CascadePriority::new(
            CascadeOrigin::Theme,
            ImportantFlag::Normal,
            spec(0, 0, 0),
            0,
        );
        let global = CascadePriority::new(
            CascadeOrigin::GlobalApp,
            ImportantFlag::Normal,
            spec(0, 0, 0),
            0,
        );
        let component = CascadePriority::new(
            CascadeOrigin::ComponentModule,
            ImportantFlag::Normal,
            spec(0, 0, 0),
            0,
        );
        let inline = CascadePriority::new(
            CascadeOrigin::Inline,
            ImportantFlag::Normal,
            spec(0, 0, 0),
            0,
        );
        assert!(system < theme);
        assert!(theme < global);
        assert!(global < component);
        assert!(component < inline);
    }

    #[test]
    fn equal_inputs_compare_equal() {
        let a = CascadePriority::new(
            CascadeOrigin::Inline,
            ImportantFlag::Normal,
            spec(1, 2, 3),
            7,
        );
        let b = CascadePriority::new(
            CascadeOrigin::Inline,
            ImportantFlag::Normal,
            spec(1, 2, 3),
            7,
        );
        assert_eq!(a, b);
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn band_values_are_ordered_by_region() {
        let authored_normal = CascadePriority::new(
            CascadeOrigin::Inline,
            ImportantFlag::Normal,
            spec(0, 0, 0),
            0,
        )
        .band();
        let authored_important = CascadePriority::new(
            CascadeOrigin::SystemDefault,
            ImportantFlag::Important,
            spec(0, 0, 0),
            0,
        )
        .band();
        let runtime = CascadePriority::new(
            CascadeOrigin::RuntimeOverride,
            ImportantFlag::Normal,
            spec(0, 0, 0),
            0,
        )
        .band();
        assert!(authored_normal < authored_important);
        assert!(authored_important < runtime);
    }
}
