//! Canonical CSS Color 4 color-space conversions to sRGB.
//!
//! Pure, deterministic math that lowers the CSS Color 4 color functions
//! (`hsl()`, `oklab()`, `oklch()`, `lab()`, `lch()`) down to gamma-encoded sRGB
//! channels in `0.0..=1.0`. Each public function returns clamped sRGB so that a
//! stylesheet's authored color always resolves to a paintable value.
//!
//! The formulas are re-derived from the CSS Color 4 specification
//! (<https://www.w3.org/TR/css-color-4/>); nothing here is copied from an
//! existing browser engine. Section references are cited inline.
//!
//! Gamut handling is a **simple per-channel clamp to `[0, 1]` after
//! conversion** (see [`clamp_srgb`]). Full CSS gamut mapping (the CSS Color 4
//! §13 binary-search chroma reduction in Oklch) is a deliberate later
//! refinement and is not implemented here.

// Color math uses the canonical single-letter component names from the spec
// (L/a/b, R/G/B, L/M/S, h/s/l); renaming them would obscure the 1:1 mapping to
// the CSS Color 4 formulas.
#![allow(clippy::many_single_char_names)]

/// sRGB electro-optical transfer: encode a linear-light channel to gamma-encoded
/// sRGB (CSS Color 4 §10.2, "gam_sRGB"). Input and output are nominally
/// `0.0..=1.0`; the sign is preserved for out-of-range inputs.
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

/// Inverse sRGB transfer: decode a gamma-encoded sRGB channel to linear light
/// (CSS Color 4 §10.2, "lin_sRGB"). Input and output are nominally `0.0..=1.0`;
/// the sign is preserved for out-of-range inputs.
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

/// Clamps one channel into `[0.0, 1.0]`.
#[must_use]
fn clamp_channel(c: f64) -> f64 {
    c.clamp(0.0, 1.0)
}

/// Clamps an sRGB triple into the `[0, 1]` gamut, per channel. This is the
/// documented simple gamut treatment (see module docs).
#[must_use]
pub fn clamp_srgb([r, g, b]: [f64; 3]) -> [f64; 3] {
    [clamp_channel(r), clamp_channel(g), clamp_channel(b)]
}

/// Converts `hsl()` to gamma-encoded sRGB (CSS Color 4 §7, "HSL to sRGB").
///
/// `hue` is in degrees (any real value; reduced mod 360), `sat` and `light` are
/// fractions in `0.0..=1.0`. HSL is defined directly on gamma-encoded sRGB, so
/// no transfer function is applied. Result is clamped to `[0, 1]`.
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

/// Linear-light sRGB triple from Oklab (Björn Ottosson's Oklab, adopted by CSS
/// Color 4 §9). `l` is `0.0..=1.0`; `a` and `b` are signed. Returns **linear**
/// sRGB (no transfer function applied), unclamped.
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

/// Converts `oklab()` to gamma-encoded sRGB (CSS Color 4 §9.2). `l` is
/// `0.0..=1.0`; `a`/`b` are signed. Result is encoded and clamped to `[0, 1]`.
#[must_use]
pub fn oklab_to_srgb(l: f64, a: f64, b: f64) -> [f64; 3] {
    let [lr, lg, lb] = oklab_to_linear_srgb(l, a, b);
    clamp_srgb([srgb_encode(lr), srgb_encode(lg), srgb_encode(lb)])
}

/// Converts `oklch()` to gamma-encoded sRGB (CSS Color 4 §9.3). `l` is
/// `0.0..=1.0`, `chroma` is `>= 0`, `hue` in degrees. Polar -> rectangular then
/// the Oklab path.
#[must_use]
pub fn oklch_to_srgb(l: f64, chroma: f64, hue_deg: f64) -> [f64; 3] {
    let h = hue_deg.to_radians();
    let a = chroma * h.cos();
    let b = chroma * h.sin();
    oklab_to_srgb(l, a, b)
}

/// Converts CIE Lab (D50) to gamma-encoded sRGB (CSS Color 4 §8 + §10.4).
///
/// `l` is the lightness in `0.0..=100.0`; `a`/`b` are signed. Steps: Lab -> XYZ
/// (D50 white) -> linear sRGB via the CSS Color 4 combined Bradford-adapted
/// matrix (D50 XYZ straight to linear sRGB) -> encode -> clamp.
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

/// Converts CIE LCH (D50) to gamma-encoded sRGB (CSS Color 4 §8.3 + §10.4). `l`
/// is `0.0..=100.0`, `chroma >= 0`, `hue` in degrees. Polar -> rectangular then
/// the Lab path.
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
        close(lch_to_srgb(50.0, 50.0, 0.0), lab_to_srgb(50.0, 50.0, 0.0), 1e-9);
    }
}
