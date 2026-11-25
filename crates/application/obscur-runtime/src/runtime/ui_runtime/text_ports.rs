#![doc = include_str!("../../../docs/api/runtime/ui_runtime/text_ports.doc/_module.md")]

use obscur_localization::locale::LanguageTag;
use obscur_text::font::FontResolver;
use obscur_text::layout::Hyphenator;
use obscur_text::shaping::{TextItemizer, TextShaper};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/text_ports.doc/set_text_shaper.md")]
    pub fn set_text_shaper(&mut self, shaper: Box<dyn TextShaper>) {
        self.scene_composer.set_text_shaper(shaper);
        self.invalidate_shaped_caches();
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/text_ports.doc/set_text_itemizer.md")]
    pub fn set_text_itemizer(&mut self, itemizer: Box<dyn TextItemizer>) {
        self.scene_composer.set_itemizer(itemizer);
        self.invalidate_shaped_caches();
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/text_ports.doc/set_font_resolver.md")]
    pub fn set_font_resolver(&mut self, resolver: Box<dyn FontResolver>) {
        self.scene_composer.set_font_resolver(resolver);
        self.invalidate_shaped_caches();
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/text_ports.doc/set_hyphenator.md")]
    pub fn set_hyphenator(&mut self, hyphenator: Box<dyn Hyphenator>) {
        self.scene_composer.set_hyphenator(hyphenator);
        self.invalidate_shaped_caches();
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/text_ports.doc/set_locale.md")]
    pub fn set_locale(&mut self, locale: &LanguageTag) {
        if &self.locale == locale {
            return;
        }
        // The shaper wants the language, not the tag: `ru-RU` and `ru-UA` shape
        // identically, and a region subtag is not something OpenType has an
        // opinion about.
        self.scene_composer
            .set_language(Some(locale.language().into()));
        if self.locale.direction() != locale.direction() {
            self.style.set_locale_direction(locale.direction());
            self.last_style = None;
        }
        self.locale = locale.clone();
        self.request_frame(crate::frame::FrameReason::WindowChange);
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/text_ports.doc/invalidate_shaped_caches.md")]
    fn invalidate_shaped_caches(&mut self) {
        // Both caches' retained results were produced by the OUTGOING port: a
        // different one shapes/wraps to different sizes — and a different size is a
        // different position, so the arranged boxes go with them. That is a layout
        // input no computed-style diff can see. Drop them (spec §4.3).
        self.layout_styles = Default::default();
        self.measure_cache.clear();
        self.arrange_cache.clear();
    }
}
