#![doc = include_str!("filter_color_matrix.doc/_module.md")]

use obscur_style::property::FilterOp;

#[doc = include_str!("filter_color_matrix.doc/color_matrix_of_ops.md")]
pub(super) fn color_matrix_of_ops(ops: &[FilterOp]) -> Option<[f32; 20]> {
    if !ops.iter().any(FilterOp::is_color_op) {
        return None;
    }
    // Running affine (linear 4×4 `m`, bias `b`), identity to start.
    let mut m: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let mut b: [f32; 4] = [0.0; 4];
    for op in ops {
        let Some((om, ob)) = color_op_matrix(op) else {
            continue; // spatial op (blur / drop-shadow) — not a color transform
        };
        // Compose op AFTER the running affine: m ← om·m, b ← om·b + ob.
        m = mat_mul(&om, &m);
        b = add4(mat_vec(&om, b), ob);
    }
    let mut out = [0.0_f32; 20];
    for (r, row) in m.iter().enumerate() {
        out[r * 4..r * 4 + 4].copy_from_slice(row);
    }
    out[16..20].copy_from_slice(&b);
    Some(out)
}

#[doc = include_str!("filter_color_matrix.doc/color_op_matrix.md")]
fn color_op_matrix(op: &FilterOp) -> Option<([[f32; 4]; 4], [f32; 4])> {
    // A diagonal RGB scale (alpha untouched) with an RGB bias.
    let rgb_diag = |k: f32, bias: f32| {
        (
            [
                [k, 0.0, 0.0, 0.0],
                [0.0, k, 0.0, 0.0],
                [0.0, 0.0, k, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            [bias, bias, bias, 0.0],
        )
    };
    // A 3×3 RGB matrix (alpha untouched, zero bias).
    let rgb_3x3 = |r: [f32; 3], g: [f32; 3], b: [f32; 3]| {
        (
            [
                [r[0], r[1], r[2], 0.0],
                [g[0], g[1], g[2], 0.0],
                [b[0], b[1], b[2], 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            [0.0; 4],
        )
    };
    Some(match *op {
        // RGB *= b.
        FilterOp::Brightness(b) => rgb_diag(b, 0.0),
        // RGB' = (RGB − 0.5)·c + 0.5  ⇒  diagonal c, bias 0.5·(1 − c).
        FilterOp::Contrast(c) => rgb_diag(c, 0.5 * (1.0 - c)),
        // A *= o (alpha-only diagonal).
        FilterOp::Opacity(o) => (
            [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, o],
            ],
            [0.0; 4],
        ),
        // channel' = (1 − 2i)·channel + i, per RGB.
        FilterOp::Invert(i) => rgb_diag(1.0 - 2.0 * i, i),
        // Saturate: the spec luminance saturation matrix (0.213/0.715/0.072).
        FilterOp::Saturate(s) => rgb_3x3(
            [0.213 + 0.787 * s, 0.715 - 0.715 * s, 0.072 - 0.072 * s],
            [0.213 - 0.213 * s, 0.715 + 0.285 * s, 0.072 - 0.072 * s],
            [0.213 - 0.213 * s, 0.715 - 0.715 * s, 0.072 + 0.928 * s],
        ),
        // Grayscale: interpolate toward the luma matrix (0.2126/0.7152/0.0722)
        // with t = 1 − amount; t == 0 (grayscale(1)) is the pure luma matrix.
        FilterOp::Grayscale(a) => {
            let t = 1.0 - a;
            rgb_3x3(
                [
                    0.2126 + 0.7874 * t,
                    0.7152 - 0.7152 * t,
                    0.0722 - 0.0722 * t,
                ],
                [
                    0.2126 - 0.2126 * t,
                    0.7152 + 0.2848 * t,
                    0.0722 - 0.0722 * t,
                ],
                [
                    0.2126 - 0.2126 * t,
                    0.7152 - 0.7152 * t,
                    0.0722 + 0.9278 * t,
                ],
            )
        }
        // Sepia: the spec sepia matrix interpolated with t = 1 − amount.
        FilterOp::Sepia(a) => {
            let t = 1.0 - a;
            rgb_3x3(
                [0.393 + 0.607 * t, 0.769 - 0.769 * t, 0.189 - 0.189 * t],
                [0.349 - 0.349 * t, 0.686 + 0.314 * t, 0.168 - 0.168 * t],
                [0.272 - 0.272 * t, 0.534 - 0.534 * t, 0.131 + 0.869 * t],
            )
        }
        // Hue-rotate: the spec hue-rotation matrix built from cos θ / sin θ and the
        // luminance constants (θ in radians).
        FilterOp::HueRotate(theta) => {
            let (s, c) = theta.sin_cos();
            rgb_3x3(
                [
                    0.213 + c * 0.787 - s * 0.213,
                    0.715 - c * 0.715 - s * 0.715,
                    0.072 - c * 0.072 + s * 0.928,
                ],
                [
                    0.213 - c * 0.213 + s * 0.143,
                    0.715 + c * 0.285 + s * 0.140,
                    0.072 - c * 0.072 - s * 0.283,
                ],
                [
                    0.213 - c * 0.213 - s * 0.787,
                    0.715 - c * 0.715 + s * 0.715,
                    0.072 + c * 0.928 + s * 0.072,
                ],
            )
        }
        // Spatial ops are not color transforms.
        FilterOp::Blur(_) | FilterOp::DropShadow(_) => return None,
    })
}

#[doc = include_str!("filter_color_matrix.doc/mat_mul.md")]
fn mat_mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0_f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[i][k] * b[k][j];
            }
            out[i][j] = sum;
        }
    }
    out
}

#[doc = include_str!("filter_color_matrix.doc/mat_vec.md")]
fn mat_vec(m: &[[f32; 4]; 4], v: [f32; 4]) -> [f32; 4] {
    let mut out = [0.0_f32; 4];
    for i in 0..4 {
        out[i] = m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2] + m[i][3] * v[3];
    }
    out
}

#[doc = include_str!("filter_color_matrix.doc/add4.md")]
fn add4(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_style::property::{BoxShadow, Length};

    #[test]
    fn color_matrix_none_when_no_color_op() {
        // No color op (empty, or blur / drop-shadow only) ⇒ no ColorMatrix (the
        // byte-identical no-color path).
        assert_eq!(color_matrix_of_ops(&[]), None);
        assert_eq!(color_matrix_of_ops(&[FilterOp::Blur(4.0)]), None);
        let shadow = BoxShadow::new(
            Length::px(1.0),
            Length::px(1.0),
            Length::px(0.0),
            Length::px(0.0),
            None,
            false,
        );
        assert_eq!(color_matrix_of_ops(&[FilterOp::DropShadow(shadow)]), None);
    }

    #[test]
    fn color_matrix_brightness_is_a_diagonal() {
        // brightness(2) ⇒ diagonal 2 on RGB, alpha 1, zero bias.
        let m = color_matrix_of_ops(&[FilterOp::Brightness(2.0)]).expect("color op");
        assert_eq!(m[0], 2.0); // r←r
        assert_eq!(m[5], 2.0); // g←g
        assert_eq!(m[10], 2.0); // b←b
        assert_eq!(m[15], 1.0); // a←a
        assert_eq!(&m[16..20], &[0.0, 0.0, 0.0, 0.0]); // no bias
        // Off-diagonal RGB entries are zero.
        assert_eq!(m[1], 0.0);
        assert_eq!(m[4], 0.0);
    }

    #[test]
    fn color_matrix_grayscale_one_is_the_luma_matrix() {
        // grayscale(1) folds to the spec luma matrix: every RGB row is the same
        // luminance triple (0.2126, 0.7152, 0.0722).
        let m = color_matrix_of_ops(&[FilterOp::Grayscale(1.0)]).expect("color op");
        for row in 0..3 {
            assert!((m[row * 4] - 0.2126).abs() < 1e-6, "row {row} r coeff");
            assert!((m[row * 4 + 1] - 0.7152).abs() < 1e-6, "row {row} g coeff");
            assert!((m[row * 4 + 2] - 0.0722).abs() < 1e-6, "row {row} b coeff");
        }
        assert_eq!(m[15], 1.0);
    }

    #[test]
    fn color_matrix_hue_rotate_90deg_builds_the_spec_matrix() {
        // hue-rotate(90°): cos = 0, sin = 1 ⇒ row0 = (0, 0, 1) exactly (the spec
        // constants collapse: 0.072 − 0 + 0.928 = 1).
        let m = color_matrix_of_ops(&[FilterOp::HueRotate(core::f32::consts::FRAC_PI_2)])
            .expect("color op");
        assert!(m[0].abs() < 1e-6, "r0c0");
        assert!(m[1].abs() < 1e-6, "r0c1");
        assert!((m[2] - 1.0).abs() < 1e-6, "r0c2 == 1");
    }

    #[test]
    fn color_matrix_chain_composes_in_author_order() {
        // grayscale(1) THEN brightness(2): grayscale collapses to luma, brightness
        // scales the result ×2, so row0 r-coeff = 2 × 0.2126 = 0.4252.
        let m = color_matrix_of_ops(&[FilterOp::Grayscale(1.0), FilterOp::Brightness(2.0)])
            .expect("color op");
        assert!((m[0] - 0.4252).abs() < 1e-5, "composed r0c0");
        assert!((m[1] - 1.4304).abs() < 1e-5, "composed r0c1 = 2×0.7152");
        // Swapping the order changes the result (brightness first, then luma): the
        // luma row absorbs the ×2 into its own weights, so r0c0 = 0.2126 (not 0.4252).
        let swapped = color_matrix_of_ops(&[FilterOp::Brightness(2.0), FilterOp::Grayscale(1.0)])
            .expect("color op");
        assert!(
            (swapped[0] - 0.4252).abs() < 1e-5,
            "brightness∘luma still ×2 in r0c0"
        );
        // The two orders differ overall (bias / alpha aside, the linear parts diverge
        // once a biased op like contrast is involved) — assert order matters via a
        // contrast+brightness pair.
        let a = color_matrix_of_ops(&[FilterOp::Contrast(2.0), FilterOp::Brightness(0.5)])
            .expect("color op");
        let b = color_matrix_of_ops(&[FilterOp::Brightness(0.5), FilterOp::Contrast(2.0)])
            .expect("color op");
        assert_ne!(a[16], b[16], "order changes the folded bias");
    }
}
