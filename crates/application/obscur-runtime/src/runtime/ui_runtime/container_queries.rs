//! Retained size query inputs from the previous arranged frame.

use obscur_geometry::Size;
use obscur_style::computed::{ChMetrics, StyleResolver};
use obscur_style::matching::{ContainerQueryEnvironment, QueryContainer};
use obscur_style::property::{LengthContext, PropertyId, PropertyValue};

use super::UiRuntime;
use crate::frame::FrameReason;
use crate::orchestration::{ShapedChMetrics, layout_style_from, root_font_size, writing_mode_of};

impl UiRuntime {
    /// Measure eligible content boxes and request a bounded restyle when the query inputs change.
    pub(super) fn update_query_containers(&mut self, viewport: Size) {
        const MAX_FOLLOW_UPS: u8 = 8;

        let style_queries = self.last_style.as_ref().is_some_and(|key| {
            key.sheets
                .iter()
                .any(obscur_style::matching::Stylesheet::uses_container_style_queries)
        });
        let mut queries = ContainerQueryEnvironment::default();
        let root_font = root_font_size(&self.computed, self.tree.root());
        for (&node, style) in &self.computed {
            let Some(PropertyValue::Name(kind)) = style
                .get(PropertyId::ContainerType)
                .map(|value| value.value())
            else {
                continue;
            };
            if !matches!(kind.as_ref(), "size" | "inline-size") {
                continue;
            }
            let Some(bounds) = self.boxes.get(&node) else {
                continue;
            };
            let layout = layout_style_from(
                style,
                viewport,
                root_font,
                Some(self.scene_composer.paragraph()),
            );
            let rect = bounds
                .used_content_rect()
                .unwrap_or_else(|| bounds.content_rect(layout.padding, layout.border));
            let both_axes = kind.as_ref() == "size";
            let vertical = layout.writing_mode.is_vertical();
            let name = register_container_names(&mut queries, node, style);
            queries.insert(
                node,
                QueryContainer {
                    name,
                    vertical,
                    width: (both_axes || !vertical).then_some(rect.size.width),
                    height: (both_axes || vertical).then_some(rect.size.height),
                },
            );
        }
        if style_queries {
            let mut requested = Vec::new();
            let mut ranges = Vec::new();
            let mut references = Vec::new();
            if let Some(key) = &self.last_style {
                for &property in key
                    .sheets
                    .iter()
                    .flat_map(|sheet| sheet.container_query_properties())
                {
                    if !requested.contains(&property) {
                        requested.push(property);
                    }
                }
                for &property in key
                    .sheets
                    .iter()
                    .flat_map(|sheet| sheet.container_query_ranges())
                {
                    if !ranges.contains(&property) {
                        ranges.push(property);
                    }
                }
                for &reference in key
                    .sheets
                    .iter()
                    .flat_map(|sheet| sheet.container_query_references())
                {
                    if !references.contains(&reference) {
                        references.push(reference);
                    }
                }
            }
            let tests_font_size = requested.contains(&PropertyId::FontSize);
            let needs_lengths = !ranges.is_empty()
                || requested
                    .iter()
                    .copied()
                    .any(PropertyId::supports_style_query_length);
            let range_root = needs_lengths.then(|| self.query_root_element()).flatten();
            let range_root_font = root_font_size(&self.computed, range_root);
            let initial_font = root_font_size(&self.computed, None);
            let range_ch = self.last_style.as_ref().is_some_and(|key| {
                key.sheets
                    .iter()
                    .any(obscur_style::matching::Stylesheet::container_ranges_use_ch)
            });
            let metrics = ShapedChMetrics::new(self.scene_composer.paragraph());
            for (node, retained) in self.tree.iter() {
                if retained.kind().is_element()
                    && let Some(style) = self.computed.get(&node)
                {
                    let name = register_container_names(&mut queries, node, style);
                    queries.insert_computed_style(node, name, style, &requested);
                    if !ranges.is_empty() {
                        queries.record_style_ranges(
                            node,
                            &ranges,
                            self.style.custom_value_parser(),
                        );
                    }
                    if !references.is_empty() {
                        queries.record_style_references(
                            node,
                            &references,
                            self.style.custom_value_parser(),
                        );
                    }
                    if needs_lengths {
                        let mut context = LengthContext::new(
                            StyleResolver::font_size_of(style),
                            range_root_font,
                            viewport.width,
                            viewport.height,
                        )
                        .with_environment(self.environment);
                        let needs_ch = range_ch || queries.style_ranges_use_ch(node);
                        if needs_ch && let Some(width) = metrics.zero_advance(style) {
                            context = context.with_ch_width(width);
                        }
                        queries.set_style_range_context(
                            node,
                            context,
                            writing_mode_of(style).is_vertical(),
                        );
                        if tests_font_size {
                            let parent = retained
                                .logical_parent()
                                .parent()
                                .and_then(|id| self.computed.get(&id));
                            let mut parent_context = context;
                            parent_context.font_size =
                                parent.map_or(initial_font, StyleResolver::font_size_of);
                            parent_context.ch_width = if needs_ch {
                                parent.and_then(|style| metrics.zero_advance(style))
                            } else {
                                None
                            };
                            if range_root == Some(node) {
                                parent_context.root_font_size = initial_font;
                            }
                            queries.set_font_size_context(node, parent_context);
                        }
                    }
                }
            }
        }
        if !queries.is_empty() {
            for (node, retained) in self.tree.iter() {
                if let Some(parent) = retained.logical_parent().parent() {
                    queries.link(node, parent);
                }
            }
        }

        if self.container_queries == queries {
            self.container_query_rounds = 0;
            return;
        }
        self.container_queries = queries;
        self.last_style = None;
        if self.container_query_rounds == MAX_FOLLOW_UPS {
            self.container_query_rounds = 0;
            return;
        }
        self.container_query_rounds += 1;
        self.scheduler.request_reason(FrameReason::UiCommit);
    }

    fn query_root_element(&self) -> Option<obscur_identity::NodeId> {
        let mut pending: Vec<_> = self.tree.root().into_iter().collect();
        while let Some(node) = pending.pop() {
            let Some(retained) = self.tree.get(node) else {
                continue;
            };
            if retained.kind().is_element() {
                return Some(node);
            }
            pending.extend(retained.child_nodes().iter().rev().copied());
        }
        None
    }
}

fn register_container_names(
    queries: &mut ContainerQueryEnvironment,
    node: obscur_identity::NodeId,
    style: &obscur_style::computed::ComputedStyle,
) -> Option<Box<str>> {
    match style
        .get(PropertyId::ContainerName)
        .map(|value| value.value())
    {
        Some(PropertyValue::Name(name)) if !name.eq_ignore_ascii_case("none") => Some(name.clone()),
        Some(PropertyValue::ContainerNames(names)) => {
            queries.set_names(node, names.clone());
            None
        }
        _ => None,
    }
}
