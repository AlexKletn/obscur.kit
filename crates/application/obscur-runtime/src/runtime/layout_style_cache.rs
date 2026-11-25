use obscur_geometry::Size;
use obscur_text::shaping::ParagraphShaper;

use super::LayoutPhase;
use crate::orchestration::{ComputedStyles, LayoutCoordinator, LayoutStyles, layout_style_from};

#[derive(Default)]
pub(crate) struct LayoutStyleCache {
    mapped: LayoutStyles,
    lengths: Option<(Size, f32)>,
}

impl LayoutStyleCache {
    pub(super) fn prepare(
        &mut self,
        layout: &LayoutCoordinator,
        computed: &ComputedStyles,
        viewport: Size,
        root_font: f32,
        paragraph: &ParagraphShaper,
        phase: &LayoutPhase,
    ) -> LayoutStyles {
        static REBUILD: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if *REBUILD.get_or_init(|| {
            std::env::var_os("OBSCUR_FULL_LAYOUT_STYLE_MAP").is_some_and(|v| v == "1")
        }) {
            return layout.layout_styles_from(computed, viewport, root_font, Some(paragraph));
        }
        let lengths = (viewport, root_font);
        if let LayoutPhase::Invalidated(dirty) = phase
            && self.lengths == Some(lengths)
            && self.mapped.len() == computed.len()
        {
            for (&node, _) in dirty.iter() {
                if let Some(style) = computed.get(&node) {
                    self.mapped.insert(
                        node,
                        layout_style_from(style, viewport, root_font, Some(paragraph)),
                    );
                }
            }
        } else {
            self.mapped = layout.layout_styles_from(computed, viewport, root_font, Some(paragraph));
            self.lengths = Some(lengths);
        }
        // Named grid placement, image sizes, and virtual spacers patch the
        // working map. The retained copy contains only the CSS conversion.
        self.mapped.clone()
    }
}

#[cfg(test)]
mod tests {
    use obscur_identity::NodeId;
    use obscur_layout::invalidation::{LayoutInvalidation, LayoutInvalidationReason};
    use obscur_style::computed::ComputedValue;
    use obscur_style::property::{Length, PropertyId, PropertyValue};

    use super::*;
    use crate::orchestration::SceneCoordinator;

    #[test]
    fn retained_layout_styles_match_a_full_map_after_resize_and_width_change() {
        let mut cache = LayoutStyleCache::default();
        let layout = LayoutCoordinator::new();
        let scene = SceneCoordinator::new();
        let paragraph = scene.paragraph();
        let mut computed = ComputedStyles::default();
        let fixed = NodeId::from_raw(1);
        let animated = NodeId::from_raw(2);
        let width = |length| ComputedValue::from_concrete(PropertyValue::Length(length)).unwrap();
        computed
            .entry(fixed)
            .or_default()
            .set(PropertyId::Width, width(Length::Vw(50.0)));
        computed
            .entry(animated)
            .or_default()
            .set(PropertyId::Width, width(Length::px(20.0)));
        let initial = Size::new(100.0, 100.0);
        let mut working = cache.prepare(
            &layout,
            &computed,
            initial,
            16.0,
            paragraph,
            &LayoutPhase::Full,
        );
        working.clear();

        computed
            .get_mut(&animated)
            .unwrap()
            .set(PropertyId::Width, width(Length::px(40.0)));
        let mut dirty = LayoutInvalidation::new();
        dirty.invalidate(animated, LayoutInvalidationReason::StyleChanged);
        let phase = LayoutPhase::Invalidated(dirty);
        for viewport in [initial, Size::new(200.0, 100.0)] {
            assert_eq!(
                cache.prepare(&layout, &computed, viewport, 16.0, paragraph, &phase),
                layout.layout_styles_from(&computed, viewport, 16.0, Some(paragraph)),
            );
        }
    }
}
