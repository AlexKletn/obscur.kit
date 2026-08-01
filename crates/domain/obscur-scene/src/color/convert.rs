#![doc = include_str!("convert.doc/_module.md")]
// Color math uses the canonical single-letter component names from the spec
// (L/a/b, R/G/B, L/M/S, h/s/l); renaming them would obscure the 1:1 mapping to
// the CSS Color 4 formulas.
#![allow(clippy::many_single_char_names)]

#[doc = include_str!("convert.doc/srgb_encode.md")]
#[must_use]
pub fn srgb_encode(c: f64) -> f64 {
    let sign = if c < 0.0 { -1.0 } else { 1.0 };
    let a = c.abs();
    if a <= 0.003_130_8 {
        12.92 * c
    } else {
        sign * (1.055 * a.powf(1.0 / 2.4) - 0.055)
    }
}

#[doc = include_str!("convert.doc/srgb_decode.md")]
#[must_use]
pub fn srgb_decode(c: f64) -> f64 {
    let sign = if c < 0.0 { -1.0 } else { 1.0 };
    let a = c.abs();
    if a <= 0.040_45 {
        c / 12.92
    } else {
        sign * ((a + 0.055) / 1.055).powf(2.4)
    }
}

#[doc = include_str!("convert.doc/clamp_channel.md")]
#[must_use]
fn clamp_channel(c: f64) -> f64 {
    c.clamp(0.0, 1.0)
}

#[doc = include_str!("convert.doc/clamp_srgb.md")]
#[must_use]
pub fn clamp_srgb([r, g, b]: [f64; 3]) -> [f64; 3] {
    [clamp_channel(r), clamp_channel(g), clamp_channel(b)]
}

#[doc = include_str!("convert.doc/hsl_to_srgb.md")]
#[must_use]
pub fn hsl_to_srgb(hue: f64, sat: f64, light: f64) -> [f64; 3] {
    // Normalize hue into [0, 360).
    let mut h = hue.rem_euclid(360.0);
    if h.is_nan() {
        h = 0.0;
    }
    let s = sat.clamp(0.0, 1.0);
    let l = light.clamp(0.0, 1.0);

    // CSS Color 4 §7 reference algorithm.
    let f = |n: f64| -> f64 {
        let k = (n + h / 30.0).rem_euclid(12.0);
        let a = s * l.min(1.0 - l);
        l - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)
    };
    clamp_srgb([f(0.0), f(8.0), f(4.0)])
}

#[doc = include_str!("convert.doc/oklab_to_linear_srgb.md")]
#[must_use]
fn oklab_to_linear_srgb(l: f64, a: f64, b: f64) -> [f64; 3] {
    // Oklab -> LMS' (inverse of the second Oklab matrix).
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;

    // Undo the cube-root nonlinearity.
    let l_cubed = l_ * l_ * l_;
    let m_cubed = m_ * m_ * m_;
    let s_cubed = s_ * s_ * s_;

    // LMS -> linear sRGB (inverse of the first Oklab matrix, folded to sRGB).
    let r = 4.076_741_662_1 * l_cubed - 3.307_711_591_3 * m_cubed + 0.230_969_929_2 * s_cubed;
    let g = -1.268_438_004_6 * l_cubed + 2.609_757_401_1 * m_cubed - 0.341_319_396_5 * s_cubed;
    let bl = -0.004_196_086_3 * l_cubed - 0.703_418_614_7 * m_cubed + 1.707_614_701_0 * s_cubed;
    [r, g, bl]
}

#[doc = include_str!("convert.doc/linear_srgb_to_oklab.md")]
#[must_use]
pub fn linear_srgb_to_oklab(r: f64, g: f64, b: f64) -> [f64; 3] {
    // Linear sRGB -> LMS (first Oklab matrix, folded to sRGB).
    let l = 0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b;
    let m = 0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b;
    let s = 0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b;

    // Cube-root nonlinearity (sign-preserving so out-of-gamut stays well defined).
    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    // LMS' -> Oklab (second Oklab matrix).
    let big_l = 0.210_454_255_3 * l_ + 0.793_617_785_0 * m_ - 0.004_072_046_8 * s_;
    let a = 1.977_998_495_1 * l_ - 2.428_592_205_0 * m_ + 0.450_593_709_9 * s_;
    let b_ = 0.025_904_037_1 * l_ + 0.782_771_766_2 * m_ - 0.808_675_766_0 * s_;
    [big_l, a, b_]
}

#[doc = include_str!("convert.doc/oklab_to_srgb.md")]
#[must_use]
pub fn oklab_to_srgb(l: f64, a: f64, b: f64) -> [f64; 3] {
    let [lr, lg, lb] = oklab_to_linear_srgb(l, a, b);
    clamp_srgb([srgb_encode(lr), srgb_encode(lg), srgb_encode(lb)])
}

#[doc = include_str!("convert.doc/oklch_to_srgb.md")]
#[must_use]
pub fn oklch_to_srgb(l: f64, chroma: f64, hue_deg: f64) -> [f64; 3] {
    let h = hue_deg.to_radians();
    let a = chroma * h.cos();
    let b = chroma * h.sin();
    oklab_to_srgb(l, a, b)
}

#[doc = include_str!("convert.doc/lab_to_srgb.md")]
#[must_use]
pub fn lab_to_srgb(l: f64, a: f64, b: f64) -> [f64; 3] {
    // CSS Color 4 §8.1 constants.
    const KAPPA: f64 = 24_389.0 / 27.0; // κ
    const EPSILON: f64 = 216.0 / 24_389.0; // ε

    // D50 reference white (CSS Color 4 §8.1): from the D50 chromaticity used by
    // Lab, Xn = 0.3457/0.3585, Yn = 1, Zn = (1 - 0.3457 - 0.3585)/0.3585.
    const XN: f64 = 0.3457 / 0.3585;
    const YN: f64 = 1.0;
    const ZN: f64 = (1.0 - 0.3457 - 0.3585) / 0.3585;

    // Lab -> XYZ (D50).
    let fy = (l + 16.0) / 116.0;
    let fx = fy + a / 500.0;
    let fz = fy - b / 200.0;

    let finv = |t: f64| -> f64 {
        let t3 = t * t * t;
        if t3 > EPSILON {
            t3
        } else {
            (116.0 * t - 16.0) / KAPPA
        }
    };

    let xr = finv(fx);
    // The y channel uses the direct lightness branch (CSS Color 4 §8.1).
    let yr = if l > KAPPA * EPSILON {
        let f = (l + 16.0) / 116.0;
        f * f * f
    } else {
        l / KAPPA
    };
    let zr = finv(fz);

    let x = xr * XN;
    let y = yr * YN;
    let z = zr * ZN;

    // D50 XYZ -> linear sRGB (CSS Color 4 §10.4: the Bradford-adapted D50->D65
    // chromatic adaptation folded into the XYZ-to-linear-sRGB matrix).
    let lr = 3.134_135_956_9 * x - 1.617_386_332_1 * y - 0.490_661_946_0 * z;
    let lg = -0.978_768_390_0 * x + 1.916_141_451_9 * y + 0.033_454_032_6 * z;
    let lb = 0.071_945_363_3 * x - 0.228_991_422_6 * y + 1.405_242_752_8 * z;

    clamp_srgb([srgb_encode(lr), srgb_encode(lg), srgb_encode(lb)])
}

#[doc = include_str!("convert.doc/lch_to_srgb.md")]
#[must_use]
pub fn lch_to_srgb(l: f64, chroma: f64, hue_deg: f64) -> [f64; 3] {
    let h = hue_deg.to_radians();
    let a = chroma * h.cos();
    let b = chroma * h.sin();
    lab_to_srgb(l, a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts two triples are equal within `eps` per channel.
    fn close(got: [f64; 3], want: [f64; 3], eps: f64) {
        for i in 0..3 {
            assert!(
                (got[i] - want[i]).abs() <= eps,
                "channel {i}: got {got:?} want {want:?} (eps {eps})",
            );
        }
    }

    #[test]
    fn srgb_transfer_round_trips() {
        // srgb -> linear -> srgb is (near) identity across the range.
        let mut c = 0.0;
        while c <= 1.0 {
            let back = srgb_encode(srgb_decode(c));
            assert!((back - c).abs() < 1e-9, "c={c} back={back}");
            c += 0.01;
        }
    }

    #[test]
    fn hsl_primaries() {
        close(hsl_to_srgb(120.0, 1.0, 0.5), [0.0, 1.0, 0.0], 1e-6); // green
        close(hsl_to_srgb(0.0, 1.0, 0.5), [1.0, 0.0, 0.0], 1e-6); // red
        close(hsl_to_srgb(240.0, 1.0, 0.5), [0.0, 0.0, 1.0], 1e-6); // blue
        close(hsl_to_srgb(0.0, 0.0, 1.0), [1.0, 1.0, 1.0], 1e-6); // white
        close(hsl_to_srgb(0.0, 0.0, 0.0), [0.0, 0.0, 0.0], 1e-6); // black
    }

    #[test]
    fn oklab_known_conversions() {
        // CSS Color 4 §9: oklab(0.627955 0.224863 0.125846) is sRGB red.
        close(
            oklab_to_srgb(0.627_955, 0.224_863, 0.125_846),
            [1.0, 0.0, 0.0],
            1e-2,
        );
        close(oklab_to_srgb(1.0, 0.0, 0.0), [1.0, 1.0, 1.0], 1e-3); // white
        close(oklab_to_srgb(0.0, 0.0, 0.0), [0.0, 0.0, 0.0], 1e-3); // black
    }

    #[test]
    fn linear_srgb_oklab_round_trips() {
        // linear sRGB -> Oklab -> (via oklab_to_linear_srgb) linear sRGB is identity.
        for rgb in [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.25, 0.5, 0.75],
            [1.0, 1.0, 1.0],
        ] {
            let [l, a, b] = linear_srgb_to_oklab(rgb[0], rgb[1], rgb[2]);
            close(oklab_to_linear_srgb(l, a, b), rgb, 1e-6);
        }
    }

    #[test]
    fn linear_srgb_to_oklab_matches_css_red() {
        // CSS Color 4 §9: sRGB red is oklab(0.627955 0.224863 0.125846). Feed the
        // LINEAR-light red (linear sRGB of gamma 1.0 is 1.0) and expect that L,a,b.
        let [l, a, b] = linear_srgb_to_oklab(1.0, 0.0, 0.0);
        close([l, a, b], [0.627_955, 0.224_863, 0.125_846], 1e-3);
    }

    #[test]
    fn oklch_known_conversions() {
        // oklch(0.62796 0.25768 29.234) ~= sRGB red.
        close(
            oklch_to_srgb(0.627_96, 0.257_68, 29.234),
            [1.0, 0.0, 0.0],
            1e-2,
        );
        // oklch(0.86644 0.29483 142.5) ~= sRGB green.
        close(
            oklch_to_srgb(0.866_44, 0.294_83, 142.5),
            [0.0, 1.0, 0.0],
            1e-2,
        );
    }

    #[test]
    fn lab_known_conversions() {
        close(lab_to_srgb(100.0, 0.0, 0.0), [1.0, 1.0, 1.0], 1e-3); // white
        close(lab_to_srgb(0.0, 0.0, 0.0), [0.0, 0.0, 0.0], 1e-3); // black

        // A mid reddish color; pinned from these functions (self-consistent) and
        // sanity-checked R > G > B and in-gamut.
        let mid = lab_to_srgb(50.0, 40.0, 30.0);
        close(mid, [0.734_2, 0.344_2, 0.276_3], 1e-2);
        assert!(mid[0] > mid[1] && mid[1] > mid[2], "R>G>B: {mid:?}");
        for c in mid {
            assert!((0.0..=1.0).contains(&c), "in gamut: {c}");
        }
    }

    #[test]
    fn lch_matches_lab() {
        // lch(50 50 0) has a = 50, b = 0, i.e. lab(50 50 0).
        close(
            lch_to_srgb(50.0, 50.0, 0.0),
            lab_to_srgb(50.0, 50.0, 0.0),
            1e-9,
        );
    }
}
