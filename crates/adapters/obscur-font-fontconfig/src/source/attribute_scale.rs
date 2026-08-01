#![doc = include_str!("attribute_scale.doc/_module.md")]

use obscur_text::font::FontStretch;
use obscur_text::span::{FontSlant, FontWeight};

#[doc = include_str!("attribute_scale.doc/WEIGHT_ANCHORS.md")]
const WEIGHT_ANCHORS: &[(i32, u16)] = &[
    (0, 100),    // FC_WEIGHT_THIN
    (40, 200),   // FC_WEIGHT_EXTRALIGHT
    (50, 300),   // FC_WEIGHT_LIGHT
    (55, 350),   // FC_WEIGHT_DEMILIGHT
    (75, 380),   // FC_WEIGHT_BOOK — CSS names no "book"; 380 is its conventional slot
    (80, 400),   // FC_WEIGHT_REGULAR
    (100, 500),  // FC_WEIGHT_MEDIUM
    (180, 600),  // FC_WEIGHT_DEMIBOLD
    (200, 700),  // FC_WEIGHT_BOLD
    (205, 800),  // FC_WEIGHT_EXTRABOLD
    (210, 900),  // FC_WEIGHT_BLACK
    (215, 1000), // FC_WEIGHT_EXTRABLACK
];

#[doc = include_str!("attribute_scale.doc/css_weight.md")]
pub(super) fn css_weight(fc_weight: i32) -> FontWeight {
    let first = WEIGHT_ANCHORS[0];
    let last = WEIGHT_ANCHORS[WEIGHT_ANCHORS.len() - 1];
    if fc_weight <= first.0 {
        return FontWeight::new(first.1);
    }
    if fc_weight >= last.0 {
        return FontWeight::new(last.1);
    }
    for pair in WEIGHT_ANCHORS.windows(2) {
        let (lo_fc, lo_css) = pair[0];
        let (hi_fc, hi_css) = pair[1];
        if fc_weight >= lo_fc && fc_weight <= hi_fc {
            let span = hi_fc - lo_fc;
            if span == 0 {
                return FontWeight::new(lo_css);
            }
            let t = f32::from(u16::try_from(fc_weight - lo_fc).unwrap_or(0)) / span as f32;
            let css = f32::from(lo_css) + t * (f32::from(hi_css) - f32::from(lo_css));
            // Anchors bound the result to `100..=1000`, so the cast neither
            // truncates meaningfully nor loses a sign.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            return FontWeight::new(css.round() as u16);
        }
    }
    FontWeight::NORMAL
}

#[doc = include_str!("attribute_scale.doc/css_slant.md")]
pub(super) fn css_slant(fc_slant: i32) -> FontSlant {
    match fc_slant {
        i32::MIN..=0 => FontSlant::Normal,
        1..=109 => FontSlant::Italic,
        _ => FontSlant::Oblique,
    }
}

#[doc = include_str!("attribute_scale.doc/css_stretch.md")]
pub(super) fn css_stretch(fc_width: i32) -> FontStretch {
    const STEPS: &[FontStretch] = &[
        FontStretch::UltraCondensed,
        FontStretch::ExtraCondensed,
        FontStretch::Condensed,
        FontStretch::SemiCondensed,
        FontStretch::Normal,
        FontStretch::SemiExpanded,
        FontStretch::Expanded,
        FontStretch::ExtraExpanded,
        FontStretch::UltraExpanded,
    ];
    let width = fc_width as f32;
    STEPS
        .iter()
        .copied()
        .min_by(|a, b| {
            (a.percentage() - width)
                .abs()
                .total_cmp(&(b.percentage() - width).abs())
        })
        .unwrap_or(FontStretch::Normal)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every anchor must round-trip exactly; an interpolation that drifted at the
    /// anchors would mislabel the common cuts, which are all *at* anchors.
    #[test]
    fn every_fontconfig_anchor_maps_to_its_css_weight() {
        for (fc, css) in WEIGHT_ANCHORS {
            assert_eq!(
                css_weight(*fc),
                FontWeight::new(*css),
                "fontconfig weight {fc} must be CSS {css}"
            );
        }
    }

    /// The bug this table exists to prevent: fontconfig's REGULAR is 80, and a
    /// reader that took the number at face value would call it CSS 80 — thinner
    /// than THIN.
    #[test]
    fn regular_is_400_not_80() {
        assert_eq!(css_weight(80), FontWeight::NORMAL);
        assert_eq!(css_weight(200), FontWeight::BOLD);
    }

    #[test]
    fn weight_interpolates_between_anchors_and_clamps_outside_them() {
        // Midway between REGULAR (80 → 400) and MEDIUM (100 → 500).
        assert_eq!(css_weight(90), FontWeight::new(450));
        assert_eq!(css_weight(-50), FontWeight::new(100));
        assert_eq!(css_weight(9999), FontWeight::new(1000));
    }

    #[test]
    fn slant_reads_the_fontconfig_constants() {
        assert_eq!(css_slant(0), FontSlant::Normal);
        assert_eq!(css_slant(100), FontSlant::Italic);
        assert_eq!(css_slant(110), FontSlant::Oblique);
    }

    /// fontconfig reports `63` for the width CSS names `62.5%`.
    #[test]
    fn stretch_snaps_fontconfigs_whole_percents_to_the_named_steps() {
        assert_eq!(css_stretch(100), FontStretch::Normal);
        assert_eq!(css_stretch(63), FontStretch::ExtraCondensed);
        assert_eq!(css_stretch(87), FontStretch::SemiCondensed);
        assert_eq!(css_stretch(200), FontStretch::UltraExpanded);
    }
}
