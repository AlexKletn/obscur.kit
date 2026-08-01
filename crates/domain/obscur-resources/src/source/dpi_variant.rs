#![doc = include_str!("dpi_variant.doc/_module.md")]

#[doc = include_str!("dpi_variant.doc/MAX_DENSITY.md")]
const MAX_DENSITY: u32 = 3;

#[doc = include_str!("dpi_variant.doc/dpi_variants.md")]
#[must_use]
pub fn dpi_variants(path: &str, scale: f32) -> Vec<(Box<str>, f32)> {
    let base: (Box<str>, f32) = (path.into(), 1.0);
    let requested = requested_density(scale);
    if requested <= 1 {
        return vec![base];
    }
    let mut candidates = Vec::with_capacity(requested as usize);
    for density in (2..=requested).rev() {
        #[allow(clippy::cast_precision_loss)]
        candidates.push((variant_path(path, density), density as f32));
    }
    candidates.push(base);
    candidates
}

#[doc = include_str!("dpi_variant.doc/requested_density.md")]
#[must_use]
pub fn requested_density(scale: f32) -> u32 {
    if !scale.is_finite() || scale <= 1.0 {
        return 1;
    }
    // `ceil` never under-resolves; the cap keeps the ladder to densities that are
    // actually authored. The cast is sound: `scale > 1.0` and finite, and the
    // `min` clamps the result to `2..=MAX_DENSITY`.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ceiled = scale.ceil() as u32;
    ceiled.min(MAX_DENSITY)
}

#[doc = include_str!("dpi_variant.doc/variant_path.md")]
fn variant_path(path: &str, density: u32) -> Box<str> {
    let suffix = format!("@{density}x");
    match extension_dot(path) {
        Some(dot) => format!("{}{}{}", &path[..dot], suffix, &path[dot..]).into(),
        None => format!("{path}{suffix}").into(),
    }
}

#[doc = include_str!("dpi_variant.doc/extension_dot.md")]
fn extension_dot(path: &str) -> Option<usize> {
    let segment_start = path.rfind(['/', '\\']).map_or(0, |slash| slash + 1);
    let segment = &path[segment_start..];
    // `rfind` takes the last dot and index 0 is skipped, so this agrees with
    // `std::path::Path::extension`: `sprite.tar.gz` extends `.gz`, and the
    // dotfile `.hidden` has no extension at all.
    segment
        .rfind('.')
        .filter(|&dot| dot > 0)
        .map(|dot| segment_start + dot)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(candidates: &[(Box<str>, f32)]) -> Vec<&str> {
        candidates.iter().map(|(p, _)| p.as_ref()).collect()
    }

    #[test]
    fn scale_one_asks_only_for_the_authored_path() {
        let ladder = dpi_variants("logo.png", 1.0);
        assert_eq!(paths(&ladder), vec!["logo.png"]);
        assert_eq!(ladder[0].1, 1.0);
    }

    #[test]
    fn scale_two_prefers_2x_then_falls_back() {
        let ladder = dpi_variants("logo.png", 2.0);
        assert_eq!(paths(&ladder), vec!["logo@2x.png", "logo.png"]);
        assert_eq!(ladder[0].1, 2.0);
        assert_eq!(ladder[1].1, 1.0);
    }

    #[test]
    fn scale_three_walks_down_every_density() {
        let ladder = dpi_variants("logo.png", 3.0);
        assert_eq!(
            paths(&ladder),
            vec!["logo@3x.png", "logo@2x.png", "logo.png"]
        );
        assert_eq!(
            ladder.iter().map(|(_, s)| *s).collect::<Vec<_>>(),
            vec![3.0, 2.0, 1.0]
        );
    }

    #[test]
    fn fractional_scale_rounds_up_and_never_under_resolves() {
        assert_eq!(
            paths(&dpi_variants("logo.png", 1.5)),
            vec!["logo@2x.png", "logo.png"]
        );
        assert_eq!(
            paths(&dpi_variants("logo.png", 2.5)),
            vec!["logo@3x.png", "logo@2x.png", "logo.png"]
        );
    }

    #[test]
    fn density_is_capped_at_3x() {
        assert_eq!(
            paths(&dpi_variants("logo.png", 8.0)),
            vec!["logo@3x.png", "logo@2x.png", "logo.png"]
        );
    }

    #[test]
    fn non_finite_or_degenerate_scale_asks_only_for_the_base() {
        assert_eq!(paths(&dpi_variants("logo.png", f32::NAN)), vec!["logo.png"]);
        assert_eq!(
            paths(&dpi_variants("logo.png", f32::INFINITY)),
            vec!["logo.png"]
        );
        assert_eq!(paths(&dpi_variants("logo.png", 0.0)), vec!["logo.png"]);
        assert_eq!(paths(&dpi_variants("logo.png", -2.0)), vec!["logo.png"]);
    }

    #[test]
    fn suffix_goes_before_the_extension_in_the_final_segment_only() {
        assert_eq!(
            paths(&dpi_variants("assets/v1.2/logo.png", 2.0)),
            vec!["assets/v1.2/logo@2x.png", "assets/v1.2/logo.png"]
        );
        assert_eq!(
            paths(&dpi_variants("assets/v1.2/logo", 2.0)),
            vec!["assets/v1.2/logo@2x", "assets/v1.2/logo"]
        );
    }

    /// The suffix goes before the *final* extension, which is what
    /// `std::path::Path::extension` calls the extension of `sprite.tar.gz`
    /// (`"gz"`, with stem `"sprite.tar"`) — so the rule matches the platform's
    /// own notion of a file extension rather than inventing a second one.
    #[test]
    fn a_double_extension_takes_the_suffix_before_the_last_dot() {
        assert_eq!(
            paths(&dpi_variants("sprite.tar.gz", 2.0)),
            vec!["sprite.tar@2x.gz", "sprite.tar.gz"]
        );
    }

    #[test]
    fn a_dotfile_segment_is_not_an_extension() {
        assert_eq!(
            paths(&dpi_variants("assets/.hidden", 2.0)),
            vec!["assets/.hidden@2x", "assets/.hidden"]
        );
    }

    #[test]
    fn no_extension_takes_the_suffix_at_the_end() {
        assert_eq!(paths(&dpi_variants("logo", 2.0)), vec!["logo@2x", "logo"]);
    }
}
