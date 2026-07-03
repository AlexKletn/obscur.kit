//! The total-order sort key that resolves a cascade conflict:
//! [`CascadePriority`] (§4.14).

use crate::cascade::{CascadeOrigin, ImportantFlag};
use crate::selector::SelectorSpecificity;
use core::cmp::Ordering;

/// The total-order sort key used to pick the winning declaration for a property
/// (§4.14).
///
/// The greater priority wins. Priority is derived from a declaration's
/// `(origin, important, specificity, source_order)` and imposes the §4.14
/// ordering as three bands, from lowest to highest precedence:
///
/// 1. **authored normal** declarations, ordered among themselves by origin
///    (`SystemDefault < Theme < GlobalApp < ComponentModule < Inline`);
/// 2. **authored `!important`** declarations, in the same origin order — this
///    whole band sits above *every* authored normal declaration, so an
///    important Theme declaration beats a normal Inline one;
/// 3. **runtime-override** declarations, which sit above all authored CSS,
///    normal or important; within this top band an `!important` runtime
///    override outranks a normal one.
///
/// These three regions are captured by a single [`band`](Self::band) integer.
/// The complete ordering key is the tuple `(band, specificity, source_order)`:
/// within one band (which pins down a single `(origin, important)` pairing) a
/// higher [`SelectorSpecificity`] wins, and on equal specificity a later
/// `source_order` wins.
///
/// Because [`band`](Self::band) is a bijection over the valid
/// `(origin, important)` inputs, ordering by that tuple is consistent with
/// equality: two priorities compare equal exactly when their band, specificity
/// and source order all match.
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
    /// Builds a priority from a declaration's cascade inputs.
    ///
    /// `source_order` is the declaration's position within its origin stream;
    /// larger means later, and later wins ties within a band.
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

    /// Computes the precedence band from `(origin, important)` (§4.14).
    ///
    /// * The runtime-override layer is the top band regardless of importance;
    ///   within it, `!important` ranks above normal.
    /// * Otherwise an authored `!important` declaration lands in a band above
    ///   all authored normal declarations.
    /// * Otherwise an authored normal declaration's band is its origin rank.
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

    /// Returns the precedence band this priority falls in.
    ///
    /// Higher bands always win over lower bands, irrespective of specificity or
    /// source order.
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
