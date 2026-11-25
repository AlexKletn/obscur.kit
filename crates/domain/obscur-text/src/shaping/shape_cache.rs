//! The bounded memo table of shaped paragraphs: [`ShapeCache`].

use super::style_group::StyleGroup;

use std::collections::HashMap;
use std::sync::Arc;

use crate::font::{FontAttributes, FontFace, FontFallback};
use crate::shaping::ShapedRun;
use crate::shaping::TextDirection;
use crate::shaping::paragraph_style_key::ParagraphStyleKey;

#[doc = include_str!("../../docs/api/shaping/shape_cache.doc/ShapeCache.md")]
pub(crate) struct ShapeCache {
    /// The generation being filled: everything inserted, or promoted, since the
    /// last rotation. Looked at first.
    young: Vec<StyleGroup>,
    /// The previous generation. Still served — a hit here is promoted back into
    /// [`young`](Self::young) — and dropped whole at the next rotation. This is
    /// the only thing that is ever evicted, so an entry has to go two full
    /// generations untouched to be lost.
    old: Vec<StyleGroup>,
    /// How many distinct paragraphs [`young`](Self::young) holds. Tracked rather
    /// than summed over the groups so the bound is checked in constant time.
    young_entries: usize,
    /// How many distinct paragraphs [`old`](Self::old) holds.
    old_entries: usize,
    /// The young-generation size that triggers a rotation: half the capacity the
    /// caller asked for, rounded down, so the two live generations together never
    /// exceed it.
    rotate_at: usize,
}

impl ShapeCache {
    /// An empty cache holding at most `capacity` shaped paragraphs across its two
    /// generations. A capacity below 2 is raised to 2: a generation that rotated
    /// on every insertion would keep nothing.
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            young: Vec::new(),
            old: Vec::new(),
            young_entries: 0,
            old_entries: 0,
            // Two generations share the budget, so neither alone may exceed
            // half of it — rounded DOWN, so an odd capacity is honoured rather
            // than overshot by one.
            rotate_at: capacity.max(2) / 2,
        }
    }

    #[doc = include_str!("../../docs/api/shaping/shape_cache.doc/get.md")]
    pub(crate) fn get(
        &mut self,
        text: &str,
        families: &FontFallback,
        attributes: FontAttributes,
        default_face: &FontFace,
        font_size: f32,
        base_direction: TextDirection,
    ) -> Option<Arc<[ShapedRun]>> {
        let matching = |group: &&mut StyleGroup| -> bool {
            group.style.matches(
                families,
                attributes,
                default_face,
                font_size,
                base_direction,
            )
        };
        // Falls through on a group that exists but does not hold this text: the
        // old generation may still have it.
        if let Some(runs) = self
            .young
            .iter_mut()
            .find(matching)
            .and_then(|group| group.texts.get(text))
        {
            return Some(Arc::clone(runs));
        }
        // A hit in the old generation is promoted, so a paragraph that is still
        // being looked at survives the next rotation instead of ageing out on a
        // schedule that ignores use.
        let promoted = self
            .old
            .iter_mut()
            .find(matching)
            .and_then(|group| group.texts.remove(text))?;
        self.old_entries -= 1;
        self.insert(
            text,
            families,
            attributes,
            default_face,
            font_size,
            base_direction,
            Arc::clone(&promoted),
        );
        Some(promoted)
    }

    #[doc = include_str!("../../docs/api/shaping/shape_cache.doc/insert.md")]
    // The shaping inputs are the cohesive identity of one paragraph, the same
    // list [`get`](Self::get) takes; bundling them would only move the names.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert(
        &mut self,
        text: &str,
        families: &FontFallback,
        attributes: FontAttributes,
        default_face: &FontFace,
        font_size: f32,
        base_direction: TextDirection,
        runs: Arc<[ShapedRun]>,
    ) {
        if self.young_entries >= self.rotate_at {
            self.old = std::mem::take(&mut self.young);
            self.old_entries = self.young_entries;
            self.young_entries = 0;
        }
        let matching = |group: &StyleGroup| {
            group.style.matches(
                families,
                attributes,
                default_face,
                font_size,
                base_direction,
            )
        };
        if !self.young.iter().any(matching) {
            self.young.push(StyleGroup {
                style: ParagraphStyleKey::new(
                    families,
                    attributes,
                    default_face,
                    font_size,
                    base_direction,
                ),
                texts: HashMap::new(),
            });
        }
        let group = self
            .young
            .iter_mut()
            .find(|group| matching(group))
            .expect("pushed just above when absent");
        if group.texts.insert(text.into(), runs).is_none() {
            self.young_entries += 1;
        }
    }

    /// Drops everything. Used when a port is swapped and no memo can be trusted.
    pub(crate) fn clear(&mut self) {
        self.young.clear();
        self.old.clear();
        self.young_entries = 0;
        self.old_entries = 0;
    }

    /// How many distinct paragraphs are memoized across both generations.
    pub(crate) fn len(&self) -> usize {
        self.young_entries + self.old_entries
    }
}
