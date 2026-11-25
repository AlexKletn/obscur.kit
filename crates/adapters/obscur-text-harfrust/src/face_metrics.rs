//! The [`FaceMetrics`] table carrying a registry's real per-face vertical
//! metrics to the shaper (spec §5.10).

use std::sync::Arc;

use obscur_text::font::{FontId, FontMetrics};
use obscur_text::shaping::LineMetrics;

#[doc = include_str!("../docs/api/face_metrics.doc/FaceMetrics.md")]
#[derive(Debug, Clone, Default)]
pub struct FaceMetrics {
    /// Per-face design metrics in registration order, indexed by
    /// [`FontId::index`]; `None` for a face whose real metrics could not be read.
    /// The whole table is optional so [`empty`](FaceMetrics::empty) — the state of
    /// a shaper that was handed no registry — is `const`-constructible.
    faces: Option<Arc<[Option<FontMetrics>]>>,
}

impl FaceMetrics {
    /// Creates a table from per-face metrics in registration order, where index
    /// `i` holds the metrics of [`FontId::new(i)`](FontId::new) and `None` marks a
    /// face whose real metrics are unavailable.
    #[must_use]
    pub fn new(faces: impl Into<Arc<[Option<FontMetrics>]>>) -> Self {
        Self {
            faces: Some(faces.into()),
        }
    }

    /// Creates the empty table: every lookup misses, so a shaper wired with it
    /// falls back for every font.
    #[must_use]
    pub const fn empty() -> Self {
        Self { faces: None }
    }

    /// Returns `font`'s design metrics, or `None` when the id names no face in
    /// this snapshot or that face's metrics could not be read.
    #[must_use]
    pub fn get(&self, font: FontId) -> Option<FontMetrics> {
        self.faces.as_ref()?.get(font.index()).copied().flatten()
    }

    #[doc = include_str!("../docs/api/face_metrics.doc/line_metrics.md")]
    #[must_use]
    pub fn line_metrics(&self, font: FontId, font_size: f32) -> Option<LineMetrics> {
        let metrics = self.get(font)?;
        if metrics.units_per_em == 0 {
            return None;
        }
        let scale = font_size / f32::from(metrics.units_per_em);
        Some(LineMetrics::new(
            metrics.ascent * scale,
            metrics.descent.abs() * scale,
            metrics.line_gap * scale,
        ))
    }

    /// Returns the number of faces in the snapshot, including those with no
    /// readable metrics.
    #[must_use]
    pub fn len(&self) -> usize {
        self.faces.as_ref().map_or(0, |faces| faces.len())
    }

    /// Returns whether the snapshot holds no faces at all — the state of a shaper
    /// that was never handed a registry's metrics.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> FaceMetrics {
        FaceMetrics::new(vec![
            // A face whose ascent ratio is 0.9, not the idealized 0.8.
            Some(FontMetrics::new(1000, 900.0, -300.0, 100.0)),
            None,
        ])
    }

    #[test]
    fn get_returns_registered_metrics_by_id() {
        let faces = table();
        assert_eq!(faces.get(FontId::PRIMARY).unwrap().ascent, 900.0);
        assert!(faces.get(FontId::new(1)).is_none());
        assert!(faces.get(FontId::new(2)).is_none());
    }

    #[test]
    fn line_metrics_scale_design_units_to_pixels() {
        let metrics = table().line_metrics(FontId::PRIMARY, 20.0).unwrap();
        assert!((metrics.ascent() - 18.0).abs() < 1e-4); // 900/1000 * 20
        // The font's negative descent becomes a positive magnitude.
        assert!((metrics.descent() - 6.0).abs() < 1e-4);
        assert!((metrics.line_gap() - 2.0).abs() < 1e-4);
        assert!((metrics.line_height() - 26.0).abs() < 1e-4);
    }

    #[test]
    fn missing_or_unreadable_face_has_no_line_metrics() {
        let faces = table();
        assert!(faces.line_metrics(FontId::new(1), 20.0).is_none());
        assert!(faces.line_metrics(FontId::new(9), 20.0).is_none());
    }

    #[test]
    fn zero_units_per_em_is_unscalable() {
        let faces = FaceMetrics::new(vec![Some(FontMetrics::new(0, 800.0, -200.0, 0.0))]);
        assert!(faces.line_metrics(FontId::PRIMARY, 16.0).is_none());
    }

    #[test]
    fn empty_and_default_tables_hold_no_faces() {
        for faces in [FaceMetrics::empty(), FaceMetrics::default()] {
            assert!(faces.is_empty());
            assert_eq!(faces.len(), 0);
            assert!(faces.line_metrics(FontId::PRIMARY, 16.0).is_none());
        }
    }

    #[test]
    fn a_table_of_metric_less_faces_is_not_empty() {
        // Length counts registered faces, not readable ones: two faces registered,
        // neither with metrics.
        let faces = FaceMetrics::new(vec![None, None]);
        assert_eq!(faces.len(), 2);
        assert!(!faces.is_empty());
    }
}
