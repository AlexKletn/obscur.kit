#![doc = include_str!("../../docs/api/value/transform_interp.doc/_module.md")]

use core::f32::consts::PI;

use super::lerp_f32;

/// Below this magnitude a scale factor is treated as degenerate (a collapsed
/// axis), so normalization substitutes a safe unit basis instead of dividing by
/// ~zero. Kept generous relative to `f32::EPSILON` so ordinary sub-pixel scales
/// still decompose exactly.
const DEGENERATE_SCALE: f32 = 1e-6;

/// A 2D affine matrix `[a, b, c, d, e, f]` mapping `(x, y)` to
/// `(a·x + c·y + e, b·x + d·y + f)` — the `matrix(a, b, c, d, e, f)` layout the
/// style/geometry crates use. Column 0 is `(a, b)`, column 1 is `(c, d)`, and
/// `(e, f)` is the translation.
pub type Matrix2D = [f32; 6];

#[doc = include_str!("../../docs/api/value/transform_interp.doc/Decomposed2D.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decomposed2D {
    /// The translation `(tx, ty)` (`matrix` coefficients `e`, `f`).
    pub translate: (f32, f32),
    /// The scale `(sx, sy)`. `sx` carries the reflection: it is negated when the
    /// affine's determinant is negative (a flip).
    pub scale: (f32, f32),
    /// The normalized XY shear term (the residual of making column 1 orthogonal
    /// to column 0, divided by `sy`). Used directly as the shear coefficient by
    /// [`recompose`] so the pair round-trips.
    pub skew: f32,
    /// The rotation angle in radians (`atan2` of the normalized first column).
    pub angle: f32,
}

#[doc = include_str!("../../docs/api/value/transform_interp.doc/decompose.md")]
// `a, b, c, d, e, f` are the canonical CSS `matrix()` coefficient names.
#[allow(clippy::many_single_char_names)]
#[must_use]
pub fn decompose(matrix: Matrix2D) -> Decomposed2D {
    let [a, b, c, d, e, f] = matrix;

    // Column 0 = (a, b), column 1 = (c, d), translation = (e, f).
    let translate = (e, f);

    // scaleX = length of column 0; normalize it to the rotation's first axis.
    let mut scale_x = a.hypot(b);
    let (mut n0x, mut n0y) = if scale_x <= DEGENERATE_SCALE {
        // Collapsed X axis: fall back to the identity basis so `angle` is 0.
        (1.0, 0.0)
    } else {
        (a / scale_x, b / scale_x)
    };

    // XY shear: project column 1 onto the (normalized) column 0, then subtract
    // it out so column 1 becomes orthogonal to column 0.
    let mut skew = n0x * c + n0y * d;
    let mut c1x = c - n0x * skew;
    let mut c1y = d - n0y * skew;

    // scaleY = length of the orthogonalized column 1; normalize it and the shear.
    let scale_y = c1x.hypot(c1y);
    if scale_y <= DEGENERATE_SCALE {
        // Collapsed Y axis: leave the shear at zero (division is undefined).
        skew = 0.0;
        c1x = -n0y;
        c1y = n0x;
    } else {
        c1x /= scale_y;
        c1y /= scale_y;
        skew /= scale_y;
    }

    // A negative determinant means one axis is reflected. Push the flip onto
    // scaleX (and its column) so the retained basis is a proper rotation.
    let determinant = n0x * c1y - n0y * c1x;
    if determinant < 0.0 {
        scale_x = -scale_x;
        n0x = -n0x;
        n0y = -n0y;
    }

    Decomposed2D {
        translate,
        scale: (scale_x, scale_y),
        skew,
        angle: n0y.atan2(n0x),
    }
}

#[doc = include_str!("../../docs/api/value/transform_interp.doc/interpolate.md")]
#[must_use]
pub fn interpolate(from: &Decomposed2D, to: &Decomposed2D, t: f32) -> Decomposed2D {
    // Shortest-arc angle blend: bring the endpoints within π of each other.
    let mut a0 = from.angle;
    let mut a1 = to.angle;
    if (a0 - a1).abs() > PI {
        if a0 < a1 {
            a0 += 2.0 * PI;
        } else {
            a1 += 2.0 * PI;
        }
    }

    Decomposed2D {
        translate: (
            lerp_f32(from.translate.0, to.translate.0, t),
            lerp_f32(from.translate.1, to.translate.1, t),
        ),
        scale: (
            lerp_f32(from.scale.0, to.scale.0, t),
            lerp_f32(from.scale.1, to.scale.1, t),
        ),
        skew: lerp_f32(from.skew, to.skew, t),
        angle: lerp_f32(a0, a1, t),
    }
}

#[doc = include_str!("../../docs/api/value/transform_interp.doc/recompose.md")]
#[must_use]
pub fn recompose(d: &Decomposed2D) -> Matrix2D {
    let (cos, sin) = (d.angle.cos(), d.angle.sin());
    let (sx, sy) = d.scale;
    let skew = d.skew;

    let a = sx * cos;
    let b = sx * sin;
    let c = sy * (skew * cos - sin);
    let dd = sy * (skew * sin + cos);
    [a, b, c, dd, d.translate.0, d.translate.1]
}

#[doc = include_str!("../../docs/api/value/transform_interp.doc/interpolate_matrix.md")]
#[must_use]
pub fn interpolate_matrix(from: Matrix2D, to: Matrix2D, t: f32) -> Matrix2D {
    let a = decompose(from);
    let b = decompose(to);
    recompose(&interpolate(&a, &b, t))
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDENTITY: Matrix2D = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    const EPS: f32 = 1e-4;

    fn approx(a: Matrix2D, b: Matrix2D) -> bool {
        a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() <= EPS)
    }

    fn approx_dec(a: &Decomposed2D, b: &Decomposed2D) -> bool {
        (a.translate.0 - b.translate.0).abs() <= EPS
            && (a.translate.1 - b.translate.1).abs() <= EPS
            && (a.scale.0 - b.scale.0).abs() <= EPS
            && (a.scale.1 - b.scale.1).abs() <= EPS
            && (a.skew - b.skew).abs() <= EPS
            && (a.angle - b.angle).abs() <= EPS
    }

    /// The signature test: `rotate(0°) → rotate(90°)` at `t = 0.5` must be
    /// `rotate(45°)` with scale preserved — NOT the naive matrix lerp.
    #[test]
    fn rotate_0_to_90_at_half_is_rotate_45_scale_preserved() {
        let r0 = IDENTITY; // rotate(0°)
        let r90 = [0.0, 1.0, -1.0, 0.0, 0.0, 0.0]; // rotate(90°)
        let mid = interpolate_matrix(r0, r90, 0.5);

        let s = 0.5_f32.sqrt(); // cos(45°) = sin(45°) ≈ 0.7071
        assert!(
            approx(mid, [s, s, -s, s, 0.0, 0.0]),
            "expected rotate(45°) ≈ [{s},{s},-{s},{s},0,0], got {mid:?}",
        );

        // Scale is preserved at 1 (a proper rotation), unlike the naive lerp
        // whose column length would be hypot(0.5, 0.5) ≈ 0.707 (a 30% shrink).
        let scale_x = mid[0].hypot(mid[1]);
        assert!(
            (scale_x - 1.0).abs() <= EPS,
            "scaleX must stay 1, got {scale_x}"
        );

        // And the recovered angle is exactly 45°.
        let angle = mid[1].atan2(mid[0]);
        assert!(
            (angle - PI / 4.0).abs() <= EPS,
            "angle must be 45°, got {angle} rad"
        );

        // Contrast: the naive component-wise lerp shrinks the box.
        let naive: Matrix2D = [0.5, 0.5, -0.5, 0.5, 0.0, 0.0];
        let naive_scale = naive[0].hypot(naive[1]);
        assert!(
            (naive_scale - s).abs() <= EPS,
            "naive lerp shrinks to ~0.707"
        );
    }

    #[test]
    fn interpolating_with_self_is_identity_at_any_t() {
        let m = [1.3, 0.4, -0.2, 0.9, 12.0, -5.0];
        for &t in &[0.0, 0.25, 0.5, 0.75, 1.0, 1.5, -0.3] {
            assert!(
                approx(interpolate_matrix(m, m, t), m),
                "self-interpolation must return the matrix at t={t}",
            );
        }
    }

    #[test]
    fn identity_stays_identity() {
        assert!(approx(
            interpolate_matrix(IDENTITY, IDENTITY, 0.5),
            IDENTITY
        ));
    }

    #[test]
    fn endpoints_are_exact() {
        let a = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let b = [0.0, 1.0, -1.0, 0.0, 10.0, 20.0];
        assert!(approx(interpolate_matrix(a, b, 0.0), a));
        assert!(approx(interpolate_matrix(a, b, 1.0), b));
    }

    #[test]
    fn translate_lerps_linearly() {
        let a = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0, 1.0, 40.0, 80.0];
        let mid = interpolate_matrix(a, b, 0.25);
        assert!(approx(mid, [1.0, 0.0, 0.0, 1.0, 10.0, 20.0]), "got {mid:?}");
    }

    #[test]
    fn non_uniform_scale_lerps() {
        let a = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]; // scale(1,1)
        let b = [3.0, 0.0, 0.0, 5.0, 0.0, 0.0]; // scale(3,5)
        let mid = interpolate_matrix(a, b, 0.5);
        assert!(approx(mid, [2.0, 0.0, 0.0, 3.0, 0.0, 0.0]), "got {mid:?}");
    }

    /// Shortest-arc: 10° → 350° must go through 0° (a −20° sweep), so at t=0.5
    /// the angle is 0°, NOT 180° (which the long +340° arc would give).
    #[test]
    fn shortest_arc_10_to_350_passes_through_zero() {
        let a = to_deg(10.0);
        let b = to_deg(350.0);
        let mid = interpolate_matrix(a, b, 0.5);
        let angle = mid[1].atan2(mid[0]);
        assert!(
            angle.abs() <= EPS,
            "10°→350° at 0.5 must be 0° (short arc), got {} deg",
            angle.to_degrees(),
        );
    }

    fn to_deg(deg: f32) -> Matrix2D {
        let r = deg.to_radians();
        [r.cos(), r.sin(), -r.sin(), r.cos(), 0.0, 0.0]
    }

    #[test]
    fn recompose_of_decompose_round_trips_matrix() {
        // rotate(30°) · scale(2, 3) · skew · translate — a composite affine.
        let composites: &[Matrix2D] = &[
            [1.3, 0.4, -0.2, 0.9, 12.0, -5.0],
            [2.0, 0.0, 0.0, 3.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0, 7.0, 8.0],
            // rotate(30°)·scale(2,3):
            {
                let (c, s) = (30f32.to_radians().cos(), 30f32.to_radians().sin());
                [2.0 * c, 2.0 * s, -3.0 * s, 3.0 * c, 4.0, 6.0]
            },
        ];
        for &m in composites {
            let round = recompose(&decompose(m));
            assert!(approx(round, m), "recompose(decompose({m:?})) = {round:?}");
        }
    }

    #[test]
    fn decompose_of_recompose_round_trips_components() {
        let d = Decomposed2D {
            translate: (12.0, -3.0),
            scale: (2.0, 0.5),
            skew: 0.3,
            angle: 0.7,
        };
        let round = decompose(recompose(&d));
        assert!(
            approx_dec(&round, &d),
            "decompose(recompose(d)) = {round:?}"
        );
    }

    #[test]
    fn flip_negative_determinant_is_handled() {
        // A reflection (negative determinant): scale(-1, 1) = flip X.
        let m = [-1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let d = decompose(m);
        assert!(
            d.scale.0 < 0.0,
            "flip must land on scaleX, got {:?}",
            d.scale
        );
        // Round-trips despite the flip.
        assert!(approx(recompose(&d), m));
    }

    #[test]
    fn rotate_and_scale_composite_round_trips_through_interpolation() {
        // Interpolating a composite with itself returns it (decompose∘recompose).
        let (c, s) = (40f32.to_radians().cos(), 40f32.to_radians().sin());
        let m = [1.5 * c, 1.5 * s, -2.5 * s, 2.5 * c, 3.0, 4.0];
        assert!(approx(interpolate_matrix(m, m, 0.37), m));
    }

    #[test]
    fn degenerate_zero_scale_stays_finite() {
        let m = [0.0, 0.0, 0.0, 0.0, 5.0, 6.0]; // fully collapsed linear part
        let out = interpolate_matrix(m, IDENTITY, 0.5);
        assert!(
            out.iter().all(|v| v.is_finite()),
            "degenerate blend must be finite: {out:?}"
        );
    }
}
