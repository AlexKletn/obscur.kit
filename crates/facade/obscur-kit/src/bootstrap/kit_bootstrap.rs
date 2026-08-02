#![doc = include_str!("kit_bootstrap.doc/_module.md")]

#[cfg(feature = "cache-fs")]
use std::path::PathBuf;

#[doc = include_str!("kit_bootstrap.doc/KitBootstrap.md")]
#[derive(Debug, Clone, Default)]
pub struct KitBootstrap {
    runtime_id: u64,
    #[cfg(feature = "render-cpu")]
    renderer_id: u64,
    #[cfg(feature = "cache-fs")]
    cache_root: Option<PathBuf>,
    // Spelled as the OPT-OUT rather than as `builtin_components: bool`, so the
    // `Default` this type derives is the defaults-on answer without a hand-written
    // `impl Default` that a later field could quietly diverge from.
    without_builtin_components: bool,
}

impl KitBootstrap {
    #[doc = include_str!("kit_bootstrap.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("kit_bootstrap.doc/with_runtime_id.md")]
    #[must_use]
    pub const fn with_runtime_id(mut self, id: u64) -> Self {
        self.runtime_id = id;
        self
    }

    #[doc = include_str!("kit_bootstrap.doc/with_renderer_id.md")]
    #[cfg(feature = "render-cpu")]
    #[must_use]
    pub const fn with_renderer_id(mut self, id: u64) -> Self {
        self.renderer_id = id;
        self
    }

    #[doc = include_str!("kit_bootstrap.doc/with_cache_root.md")]
    #[cfg(feature = "cache-fs")]
    #[must_use]
    pub fn with_cache_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.cache_root = Some(root.into());
        self
    }

    #[doc = include_str!("kit_bootstrap.doc/headless_host.md")]
    #[must_use]
    pub fn headless_host(&self) -> crate::headless::HeadlessHost {
        crate::headless::HeadlessHost::new(crate::identity::RuntimeId::from_raw(self.runtime_id))
    }

    #[doc = include_str!("kit_bootstrap.doc/ua_stylesheet.md")]
    #[must_use]
    pub fn ua_stylesheet(&self) -> crate::style::matching::Stylesheet {
        crate::default_stylesheet::ua_stylesheet()
    }

    #[doc = include_str!("kit_bootstrap.doc/install_default_stylesheet.md")]
    pub fn install_default_stylesheet(&self, runtime: &mut crate::runtime::UiRuntime) {
        runtime.set_default_stylesheet(self.ua_stylesheet());
    }

    #[doc = include_str!("kit_bootstrap.doc/without_builtin_components.md")]
    #[must_use]
    pub const fn without_builtin_components(mut self) -> Self {
        self.without_builtin_components = true;
        self
    }

    #[doc = include_str!("kit_bootstrap.doc/registers_builtin_components.md")]
    #[must_use]
    pub const fn registers_builtin_components(&self) -> bool {
        !self.without_builtin_components
    }

    #[doc = include_str!("kit_bootstrap.doc/install_builtin_components.md")]
    pub fn install_builtin_components(
        &self,
        runtime: &mut crate::runtime::UiRuntime,
    ) -> Result<(), crate::view::component::RegistryError> {
        if self.without_builtin_components {
            return Ok(());
        }
        super::BuiltinComponents::install(runtime)
    }

    #[doc = include_str!("kit_bootstrap.doc/default_renderer.md")]
    #[cfg(feature = "render-cpu")]
    #[must_use]
    pub fn default_renderer(&self) -> crate::render_cpu::renderer::CpuRenderer {
        crate::render_cpu::renderer::CpuRenderer::new(crate::render::RendererId::from_raw(
            self.renderer_id,
        ))
    }

    #[doc = include_str!("kit_bootstrap.doc/default_resource_loader.md")]
    #[cfg(feature = "resource-fs")]
    #[must_use]
    pub fn default_resource_loader(&self) -> crate::resource_fs::loader::FsResourceLoader {
        crate::resource_fs::loader::FsResourceLoader::new()
    }

    #[doc = include_str!("kit_bootstrap.doc/default_image_decoder.md")]
    #[cfg(feature = "resource-decode")]
    #[must_use]
    pub fn default_image_decoder(&self) -> crate::resource_decode::decoder::ImageDecoder {
        crate::resource_decode::decoder::ImageDecoder::new()
    }

    #[doc = include_str!("kit_bootstrap.doc/install_image_resources.md")]
    #[cfg(all(feature = "resource-fs", feature = "resource-decode"))]
    pub fn install_image_resources(
        &self,
        runtime: &mut crate::runtime::UiRuntime,
        sandbox_root: impl Into<std::path::PathBuf>,
    ) {
        runtime.set_resource_providers(
            std::sync::Arc::new(crate::resource_fs::loader::FsResourceLoader::with_root(
                sandbox_root,
            )),
            std::sync::Arc::new(crate::resource_decode::decoder::ImageDecoder::new()),
        );
    }

    #[doc = include_str!("kit_bootstrap.doc/install_vector_resources.md")]
    #[cfg(feature = "resource-decode")]
    pub fn install_vector_resources(&self, runtime: &mut crate::runtime::UiRuntime) {
        runtime.set_vector_decoder(std::sync::Arc::new(
            crate::resource_decode::decoder::SvgVectorizer::new(),
        ));
    }

    #[doc = include_str!("kit_bootstrap.doc/install_accessibility.md")]
    #[cfg(feature = "accesskit")]
    #[must_use]
    pub fn install_accessibility(
        &self,
        runtime: &mut crate::runtime::UiRuntime,
    ) -> std::rc::Rc<std::cell::RefCell<crate::accesskit_bridge::bridge::AccessKitBridge>> {
        let bridge = std::rc::Rc::new(std::cell::RefCell::new(
            crate::accesskit_bridge::bridge::AccessKitBridge::new(),
        ));
        runtime.set_accessibility_bridge(bridge.clone());
        bridge
    }

    #[doc = include_str!("kit_bootstrap.doc/default_cache_store.md")]
    #[cfg(feature = "cache-fs")]
    #[must_use]
    pub fn default_cache_store(&self) -> Option<crate::cache_fs::store::FsCacheStore> {
        self.cache_root
            .as_ref()
            .map(crate::cache_fs::store::FsCacheStore::new)
    }

    #[doc = include_str!("kit_bootstrap.doc/default_task_spawner.md")]
    #[cfg(feature = "task-smol")]
    #[must_use]
    pub fn default_task_spawner(&self) -> crate::task_smol::spawner::SmolSpawner {
        crate::task_smol::spawner::SmolSpawner::new()
    }

    #[doc = include_str!("kit_bootstrap.doc/install_blocking_executor.md")]
    #[cfg(feature = "task-smol")]
    pub fn install_blocking_executor(
        &self,
        runtime: &mut crate::runtime::UiRuntime,
    ) -> std::sync::Arc<crate::task_smol::spawner::SmolSpawner> {
        let spawner = std::sync::Arc::new(crate::task_smol::spawner::SmolSpawner::new());
        runtime.set_blocking_executor(spawner.clone());
        spawner
    }

    #[doc = include_str!("kit_bootstrap.doc/system_shell_provider.md")]
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn system_shell_provider(&self) -> crate::system_platform::provider::LinuxShellProvider {
        crate::system_platform::provider::LinuxShellProvider::new()
    }

    #[doc = include_str!("kit_bootstrap.doc/text_shaper_from_bytes.md")]
    #[cfg(feature = "text-harfrust")]
    #[must_use]
    pub fn text_shaper_from_bytes(
        &self,
        bytes: impl Into<std::sync::Arc<[u8]>>,
    ) -> Box<dyn crate::text::shaping::TextShaper> {
        Box::new(crate::text_harfrust::HarfrustShaper::from_bytes(bytes))
    }

    #[doc = include_str!("kit_bootstrap.doc/text_shaper_without_font.md")]
    #[cfg(feature = "text-harfrust")]
    #[must_use]
    pub fn text_shaper_without_font(
        &self,
        faces: crate::text_harfrust::FaceMetrics,
    ) -> Box<dyn crate::text::shaping::TextShaper> {
        Box::new(crate::text_harfrust::HarfrustShaper::without_font().with_faces(faces))
    }

    #[doc = include_str!("kit_bootstrap.doc/font_registry.md")]
    #[cfg(feature = "text-harfrust")]
    #[must_use]
    pub fn font_registry(&self) -> crate::text_harfrust::FontRegistry {
        crate::text_harfrust::FontRegistry::new()
    }

    #[doc = include_str!("kit_bootstrap.doc/text_itemizer.md")]
    #[cfg(feature = "text-unicode")]
    #[must_use]
    pub fn text_itemizer(&self) -> Box<dyn crate::text::shaping::TextItemizer> {
        Box::new(crate::text_unicode::UnicodeTextItemizer::new())
    }

    #[doc = include_str!("kit_bootstrap.doc/install_text_stack.md")]
    #[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
    pub fn install_text_stack(
        &self,
        runtime: &mut crate::runtime::UiRuntime,
        fonts: &super::FontSet,
    ) -> super::InstalledFonts {
        use crate::text::font::{FontAttributes, FontFamily, GenericFamily};

        #[doc = include_str!("kit_bootstrap.doc/BUNDLED_FALLBACK.md")]
        const BUNDLED_FALLBACK: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/fonts/NotoSans-Regular.ttf"
        ));

        let mut registry = crate::text_harfrust::FontRegistry::new();
        let mut handles = Vec::with_capacity(fonts.len() + 1);
        for face in fonts.faces() {
            let id = registry.register(
                face.family().clone(),
                *face.attributes(),
                std::sync::Arc::clone(face.bytes()),
            );
            handles.push((
                super::text_stack::font_handle(id),
                std::sync::Arc::clone(face.bytes()),
            ));
        }

        // Append the bundled last-resort face after the host's, unless the host
        // already offered these exact bytes. It resolves under `sans-serif`, the
        // family every default fallback chain terminates in, so an empty set still
        // covers text and a host set only reaches it for glyphs nothing else has.
        let host_has_bundled = fonts
            .faces()
            .iter()
            .any(|face| face.bytes().as_ref() == BUNDLED_FALLBACK);
        if !host_has_bundled {
            let bundled: std::sync::Arc<[u8]> = std::sync::Arc::from(BUNDLED_FALLBACK);
            let id = registry.register(
                FontFamily::generic(GenericFamily::SansSerif),
                FontAttributes::default(),
                std::sync::Arc::clone(&bundled),
            );
            // Surface it to the renderer hand-off only when the host gave no faces
            // of its own — otherwise the returned table stays exactly the host's,
            // which every renderer-side caller already depends on.
            if fonts.is_empty() {
                handles.push((super::text_stack::font_handle(id), bundled));
            }
        }

        // Taken only now: a snapshot from before the loop would be empty, and the
        // shaper would silently answer every line-metric query with the ratios.
        let shaper = crate::text_harfrust::HarfrustShaper::without_font()
            .with_faces(registry.face_metrics());
        // Per-character host fallback (§5.10): the three generic faces plus the
        // bundled one cover Latin, Cyrillic and Greek and almost nothing else, so
        // without this a CJK file name or a symbol on a key draws *nothing*. With
        // it, the first appearance of such a character costs one `fc-match`
        // lookup and the face is then kept. Only wired where fontconfig is — a
        // host without it keeps exactly the previous behaviour.
        #[cfg(feature = "font-fontconfig")]
        let registry =
            registry.with_system_fallback(std::sync::Arc::new(super::FontconfigFallback::new()));
        // The registry is cloned rather than moved: the discovery state is shared
        // between clones, so the copy kept here sees the faces the runtime's copy
        // pulls in and can hand them to the renderer.
        runtime.set_font_resolver(Box::new(registry.clone()));
        runtime.set_text_itemizer(Box::new(crate::text_unicode::UnicodeTextItemizer::new()));
        runtime.set_text_shaper(Box::new(shaper));
        super::InstalledFonts::new(handles).with_registry(registry)
    }

    #[doc = include_str!("kit_bootstrap.doc/system_font_set.md")]
    #[cfg(feature = "font-fontconfig")]
    #[must_use]
    pub fn system_font_set(&self) -> Option<super::FontSet> {
        use crate::text::font::{FontFamily, GenericFamily};

        let source = crate::font_fontconfig::source::FontconfigSource::new();
        let mut set = super::FontSet::new();
        for generic in [
            GenericFamily::SansSerif,
            GenericFamily::Serif,
            GenericFamily::Monospace,
        ] {
            let Some(face) = source.match_generic(generic) else {
                continue;
            };
            let Ok(bytes) = std::fs::read(face.path()) else {
                continue;
            };
            set.push(super::FontFaceBytes::new(
                FontFamily::generic(generic),
                *face.attributes(),
                bytes,
            ));
        }
        (!set.is_empty()).then_some(set)
    }
}

#[cfg(test)]
mod tests {
    use super::KitBootstrap;

    #[test]
    fn builds_headless_host() {
        let boot = KitBootstrap::new().with_runtime_id(7);
        let _host = boot.headless_host();
    }

    #[cfg(feature = "render-cpu")]
    #[test]
    fn default_renderer_has_no_device() {
        use crate::render::Renderer;
        let boot = KitBootstrap::new().with_renderer_id(3);
        let renderer = boot.default_renderer();
        assert_eq!(renderer.frames_rendered(), 0);
        // The default renderer is the software one: a host that asks is told, before
        // it renders anything, that the GPU-class effects will not appear (spec
        // §5.2, and the CPU/GPU effect-split policy).
        assert_eq!(
            renderer.supports_effect(crate::render::SceneEffect::Blur),
            obscur_capability::SupportLevel::Unavailable
        );
    }

    #[cfg(feature = "resource-fs")]
    #[test]
    fn builds_resource_loader() {
        let _loader = KitBootstrap::new().default_resource_loader();
    }

    #[cfg(feature = "cache-fs")]
    #[test]
    fn cache_store_is_opt_in() {
        assert!(KitBootstrap::new().default_cache_store().is_none());
        assert!(
            KitBootstrap::new()
                .with_cache_root(std::env::temp_dir())
                .default_cache_store()
                .is_some()
        );
    }

    #[cfg(feature = "task-smol")]
    #[test]
    fn builds_task_spawner() {
        // Constructing and dropping must not hang (SmolSpawner::drop joins its worker).
        let _spawner = KitBootstrap::new().default_task_spawner();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn builds_system_shell_provider() {
        let _shell = KitBootstrap::new().system_shell_provider();
    }

    /// The install is what makes accessibility a kit facility rather than
    /// something each host hand-wires. `tests/accessibility_output_path.rs` proves
    /// what the installed bridge is then handed; this proves the one call arms it.
    #[cfg(feature = "accesskit")]
    #[test]
    fn install_accessibility_arms_the_runtimes_output_path() {
        let mut runtime = crate::runtime::UiRuntime::new(
            crate::identity::RuntimeId::from_raw(1),
            crate::geometry::Size::new(100.0, 100.0),
        );
        assert!(!runtime.has_accessibility_bridge());
        let _bridge = KitBootstrap::new().install_accessibility(&mut runtime);
        assert!(
            runtime.has_accessibility_bridge(),
            "one call, and every frame publishes"
        );
    }
}
