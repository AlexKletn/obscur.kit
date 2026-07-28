//! High-level toolkit bootstrap (spec §2.9): assembles the default adapter
//! instances selected by the enabled Cargo features, so an application gets a
//! working set of backends without hand-picking individual adapter crates.

#[cfg(feature = "cache-fs")]
use std::path::PathBuf;

/// A builder that produces the toolkit's default backends.
///
/// The factory methods are gated on the Cargo features that pull in their
/// adapter crate, so this type always compiles for the active feature set. Each
/// method constructs a fresh instance — the caller owns the wiring into a
/// runtime or host (the platform event loop / GPU device live with the host).
#[derive(Debug, Clone, Default)]
pub struct KitBootstrap {
    runtime_id: u64,
    #[cfg(feature = "render-cpu")]
    renderer_id: u64,
    #[cfg(feature = "cache-fs")]
    cache_root: Option<PathBuf>,
}

impl KitBootstrap {
    /// Creates a bootstrap with default identifiers and no cache root.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the runtime identifier used for the headless host.
    #[must_use]
    pub const fn with_runtime_id(mut self, id: u64) -> Self {
        self.runtime_id = id;
        self
    }

    /// Sets the renderer identifier used for the default renderer.
    #[cfg(feature = "render-cpu")]
    #[must_use]
    pub const fn with_renderer_id(mut self, id: u64) -> Self {
        self.renderer_id = id;
        self
    }

    /// Sets the on-disk directory for the persistent cache store.
    #[cfg(feature = "cache-fs")]
    #[must_use]
    pub fn with_cache_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.cache_root = Some(root.into());
        self
    }

    /// Builds a deterministic [`HeadlessHost`](crate::headless::HeadlessHost) —
    /// a real runtime driven by a deterministic clock and executor, for
    /// reproducible, offscreen rendering and tests. Always available (no feature
    /// gate).
    #[must_use]
    pub fn headless_host(&self) -> crate::headless::HeadlessHost {
        crate::headless::HeadlessHost::new(crate::identity::RuntimeId::from_raw(self.runtime_id))
    }

    /// Returns the compiled built-in **user-agent (UA) default stylesheet** — the
    /// toolkit's neutral, overridable base component look, at the lowest cascade
    /// origin ([`CascadeOrigin::SystemDefault`](crate::style::cascade::CascadeOrigin::SystemDefault)).
    ///
    /// See [`crate::default_stylesheet`] for the raw CSS and the override contract.
    #[must_use]
    pub fn ua_stylesheet(&self) -> crate::style::matching::Stylesheet {
        crate::default_stylesheet::ua_stylesheet()
    }

    /// Installs the built-in UA default stylesheet on `runtime` as its always-first
    /// (lowest-origin) sheet, so bare built-in controls get the toolkit's base look
    /// automatically.
    ///
    /// This is purely additive and **overridable**: the UA sheet is matched below
    /// every author/theme sheet the app later installs
    /// ([`set_stylesheets`](crate::runtime::UiRuntime::set_stylesheets) / the
    /// `&[Stylesheet]` render paths), so any author rule of the same or higher
    /// specificity wins over it (§4.14). Not calling this leaves the runtime with no
    /// UA styling (byte-identical to before).
    pub fn install_default_stylesheet(&self, runtime: &mut crate::runtime::UiRuntime) {
        runtime.set_default_stylesheet(self.ua_stylesheet());
    }

    /// Builds the default software [`Renderer`](crate::render::Renderer).
    #[cfg(feature = "render-cpu")]
    #[must_use]
    pub fn default_renderer(&self) -> crate::render_cpu::renderer::CpuRenderer {
        crate::render_cpu::renderer::CpuRenderer::new(crate::render::RendererId::from_raw(
            self.renderer_id,
        ))
    }

    /// Builds the default filesystem resource loader.
    #[cfg(feature = "resource-fs")]
    #[must_use]
    pub fn default_resource_loader(&self) -> crate::resource_fs::loader::FsResourceLoader {
        crate::resource_fs::loader::FsResourceLoader::new()
    }

    /// Builds the default raster-image decoder (the `image`-crate adapter that
    /// turns encoded PNG/JPEG bytes into RGBA8 pixels + intrinsic size, spec
    /// §5.16).
    ///
    /// Pair it with [`Self::default_resource_loader`] and inject both into a
    /// [`UiRuntime`](crate::runtime::UiRuntime) via
    /// [`set_resource_providers`](crate::runtime::UiRuntime::set_resource_providers)
    /// (or use [`Self::install_image_resources`]) to make `background-image:
    /// url(...)` resolve against the filesystem.
    #[cfg(feature = "resource-decode")]
    #[must_use]
    pub fn default_image_decoder(&self) -> crate::resource_decode::decoder::ImageDecoder {
        crate::resource_decode::decoder::ImageDecoder::new()
    }

    /// Injects a filesystem image loader + the raster-image decoder into `runtime`
    /// so its `background-image: url(...)` backgrounds resolve against files under
    /// `sandbox_root` (spec §5.15 file loader, §5.16 decode, §2.7 adapters keep
    /// host reach explicit).
    ///
    /// The facade constructs the concrete adapters (a sandbox-rooted
    /// [`FsResourceLoader`](crate::resource_fs::loader::FsResourceLoader) + the
    /// [`ImageDecoder`](crate::resource_decode::decoder::ImageDecoder)) and hands
    /// them to the runtime as domain ports, so the application layer depends only
    /// on the ports — never on these adapter crates. A `url()` path resolves
    /// relative to `sandbox_root` and cannot escape it. Not calling this leaves the
    /// runtime's honest-capability default: `url()` backgrounds paint nothing.
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

    /// Injects the SVG vector decoder into `runtime` so an `<img src="...svg">`
    /// paints as vector outlines that follow the computed `color` (spec §5.17).
    ///
    /// Additive to [`install_image_resources`](Self::install_image_resources),
    /// which supplies the loader this needs and handles bitmaps: a host may want
    /// raster images without SVG, or both. Not calling it leaves the runtime's
    /// honest-capability default — an SVG `src` paints nothing rather than being
    /// rasterized behind the author's back, which would break §5.17's "SVG remains
    /// vector through the scene pipeline".
    ///
    /// The document is parsed once and cached with its `currentColor` unresolved,
    /// so a theme change re-colours it without reparsing.
    #[cfg(feature = "resource-decode")]
    pub fn install_vector_resources(&self, runtime: &mut crate::runtime::UiRuntime) {
        runtime.set_vector_decoder(std::sync::Arc::new(
            crate::resource_decode::decoder::SvgVectorizer::new(),
        ));
    }

    /// Injects the AccessKit accessibility bridge into `runtime` so every frame
    /// publishes its semantic tree, and returns the shared bridge the host
    /// publishes *from* (spec §7.1).
    ///
    /// This is the §7.1 output path's kit half, and it is here for the same reason
    /// [`CssPipeline`](super::CssPipeline) is: joining the runtime's semantic
    /// projection to a platform adapter is fixed plumbing every host needs and
    /// none should hand-write. The facade constructs the concrete adapter and hands
    /// the runtime a domain port, so the application layer depends only on
    /// [`AccessibilityBridge`](crate::accessibility::port::AccessibilityBridge) —
    /// never on this adapter crate. From here on, each frame that changes the
    /// retained tree or moves focus converts a fresh AccessKit `TreeUpdate` into
    /// the returned bridge; nothing in the application ever pushes it.
    ///
    /// # What the host must still supply
    ///
    /// **This does not by itself reach a screen reader**, and the returned handle
    /// is the reason: AccessKit's platform half needs an OS window, which this kit
    /// never owns. A host completes the path with the platform adapter for its
    /// windowing layer (`accesskit_winit`, `accesskit_unix`, …) and, on each of its
    /// own event-loop turns:
    ///
    /// - feeds [`last_tree_update`](crate::accesskit_bridge::bridge::AccessKitBridge::last_tree_update)
    ///   to that adapter — re-reading it whenever
    ///   [`is_dirty`](crate::accesskit_bridge::bridge::AccessKitBridge::is_dirty)
    ///   or a frame ran;
    /// - drains [`take_announcements`](crate::accesskit_bridge::bridge::AccessKitBridge::take_announcements)
    ///   into the platform's live-region events;
    /// - translates each inbound AccessKit `ActionRequest` with
    ///   [`ActionConverter::from_accesskit`](crate::accesskit_bridge::conversion::ActionConverter::from_accesskit)
    ///   and enqueues it with
    ///   [`push_action`](crate::accesskit_bridge::bridge::AccessKitBridge::push_action).
    ///
    /// That last one is only half of §7.1's "accessibility actions enter the same
    /// command/event model as ordinary input": the runtime does **not** drain
    /// [`poll_actions`](crate::accessibility::port::AccessibilityBridge::poll_actions),
    /// so an action an assistive technology requests reaches the bridge and stops
    /// there. The inbound path is unbuilt; this installs the outbound one.
    ///
    /// Not calling this leaves the runtime's honest-capability default: no semantic
    /// tree is built at all.
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

    /// Builds the default filesystem persistent cache store, if a cache root was
    /// configured (otherwise `None` — persistence is opt-in).
    #[cfg(feature = "cache-fs")]
    #[must_use]
    pub fn default_cache_store(&self) -> Option<crate::cache_fs::store::FsCacheStore> {
        self.cache_root
            .as_ref()
            .map(crate::cache_fs::store::FsCacheStore::new)
    }

    /// Builds the default task spawner (a smol-backed executor).
    ///
    /// The returned object satisfies **two** ports: [`Spawner`](crate::task::Spawner),
    /// for `Send` futures on its own worker thread, and
    /// [`BlockingExecutor`](crate::task::BlockingExecutor), for blocking jobs on
    /// smol's blocking pool. Only the second has a consumer inside the kit —
    /// [`install_blocking_executor`](Self::install_blocking_executor) is the
    /// one-call way to reach it; nothing in the runtime accepts a `Spawner`, so
    /// that half is for the host's own work.
    #[cfg(feature = "task-smol")]
    #[must_use]
    pub fn default_task_spawner(&self) -> crate::task_smol::spawner::SmolSpawner {
        crate::task_smol::spawner::SmolSpawner::new()
    }

    /// Installs the default **threaded** blocking-work facility on `runtime`, so
    /// its resource loads run on another thread instead of on the frame
    /// (spec §1.5 "resource loading" as potential parallel work, §3.3 blocking
    /// work is opt-in and capability-dependent).
    ///
    /// This is the one call that turns §1.5's resource-loading parallelism on. It
    /// is **opt-in by design, not by omission**: §3.3 makes blocking work a
    /// capability, and the runtime without it is not degraded — the load simply
    /// runs on the frame thread and the image is ready that same frame.
    ///
    /// # What changes when you call this
    ///
    /// A `url()` image no longer paints on the frame that first references it.
    /// The load is handed to the pool, comes back as a typed message, and is
    /// committed at the next frame's synchronization point — so it paints one
    /// frame later. A host that renders exactly one frame and reads back the
    /// pixels (an offscreen screenshot) must therefore **not** install this, or
    /// must render again after
    /// [`poll_resource_tasks`](crate::runtime::UiRuntime::poll_resource_tasks)
    /// reports the load landed. A host with a live event loop should also call
    /// `poll_resource_tasks` from it, which is what asks for a frame when a load
    /// lands while the UI is idle; a host that renders continuously does not need
    /// to, because the frame commits results itself.
    ///
    /// # Determinism
    ///
    /// Do not install this on a headless host.
    /// [`HeadlessHost`](crate::headless::HeadlessHost) installs a
    /// `DeterministicExecutor` on the same port and would be overwritten, taking
    /// §7.3/§7.4's reproducibility with it: the deterministic executor's whole
    /// job is that a load lands when `run_tasks` says so and never before.
    ///
    /// Returns the spawner so the host can keep using its
    /// [`Spawner`](crate::task::Spawner) half; the runtime holds its own
    /// reference, so dropping the returned handle does not disarm the runtime.
    #[cfg(feature = "task-smol")]
    pub fn install_blocking_executor(
        &self,
        runtime: &mut crate::runtime::UiRuntime,
    ) -> std::sync::Arc<crate::task_smol::spawner::SmolSpawner> {
        let spawner = std::sync::Arc::new(crate::task_smol::spawner::SmolSpawner::new());
        runtime.set_blocking_executor(spawner.clone());
        spawner
    }

    /// Builds the platform shell provider. On Linux this shells out to
    /// `xdg-open`; the dialog and notification providers are obtained the same
    /// way from `crate::system_platform::provider`.
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn system_shell_provider(&self) -> crate::system_platform::provider::LinuxShellProvider {
        crate::system_platform::provider::LinuxShellProvider::new()
    }

    /// Builds the real [`HarfrustShaper`](crate::text_harfrust::HarfrustShaper)
    /// wired to a single host font from its raw `bytes`.
    ///
    /// Install the result on a runtime with
    /// [`UiRuntime::set_text_shaper`](crate::runtime::UiRuntime::set_text_shaper),
    /// using the **same** bytes handed to the renderer's `set_font`, so the
    /// shaper's emitted glyph indices align with the font the renderer rasterizes.
    /// The shaper is returned boxed as the domain `TextShaper` port, so callers
    /// depend only on the port, not the concrete adapter.
    #[cfg(feature = "text-harfrust")]
    #[must_use]
    pub fn text_shaper_from_bytes(
        &self,
        bytes: impl Into<std::sync::Arc<[u8]>>,
    ) -> Box<dyn crate::text::shaping::TextShaper> {
        Box::new(crate::text_harfrust::HarfrustShaper::from_bytes(bytes))
    }

    /// Builds a font-less real
    /// [`HarfrustShaper`](crate::text_harfrust::HarfrustShaper) that shapes each
    /// itemized run against the face bytes its
    /// [`FontResolver`](crate::text::font::FontResolver) resolved
    /// ([`FontSource::Memory`](crate::text::font::FontSource::Memory)).
    ///
    /// Pair it with [`Self::font_registry`] and [`Self::text_itemizer`] on a
    /// runtime ([`set_text_shaper`](crate::runtime::UiRuntime::set_text_shaper),
    /// [`set_font_resolver`](crate::runtime::UiRuntime::set_font_resolver),
    /// [`set_text_itemizer`](crate::runtime::UiRuntime::set_text_itemizer)) to
    /// wire the full multi-font fallback stack: itemize → resolve by coverage →
    /// shape per item.
    ///
    /// `faces` is that same registry's
    /// [`face_metrics`](crate::text_harfrust::FontRegistry::face_metrics), taken
    /// after every face is registered: a shaper is asked for line metrics by
    /// [`FontId`](crate::text::font::FontId) alone, so without the table it would
    /// shape real glyphs into line boxes sized by idealized ratios. Pass
    /// [`FaceMetrics::empty`](crate::text_harfrust::FaceMetrics::empty) only when
    /// there is deliberately no registry — line boxes are then ratio-sized, which
    /// [`HarfrustShaper::idealized_line_metrics`](crate::text_harfrust::HarfrustShaper::idealized_line_metrics)
    /// reports.
    #[cfg(feature = "text-harfrust")]
    #[must_use]
    pub fn text_shaper_without_font(
        &self,
        faces: crate::text_harfrust::FaceMetrics,
    ) -> Box<dyn crate::text::shaping::TextShaper> {
        Box::new(crate::text_harfrust::HarfrustShaper::without_font().with_faces(faces))
    }

    /// Builds an empty [`FontRegistry`](crate::text_harfrust::FontRegistry) — the
    /// real [`FontResolver`](crate::text::font::FontResolver) that picks a face
    /// from a fallback chain by true `cmap` coverage.
    ///
    /// Register faces on it (each `register` returns the
    /// [`FontId`](crate::text::font::FontId) assigned in order `0, 1, …`), then
    /// install it with
    /// [`set_font_resolver`](crate::runtime::UiRuntime::set_font_resolver). Use
    /// the **same** id↔bytes assignment on the renderer's `register_font` so
    /// scene run handles line up with the renderer's fonts, and hand its
    /// [`face_metrics`](crate::text_harfrust::FontRegistry::face_metrics) to
    /// [`Self::text_shaper_without_font`] so the faces also size their own line
    /// boxes.
    #[cfg(feature = "text-harfrust")]
    #[must_use]
    pub fn font_registry(&self) -> crate::text_harfrust::FontRegistry {
        crate::text_harfrust::FontRegistry::new()
    }

    /// Builds the real
    /// [`UnicodeTextItemizer`](crate::text_unicode::UnicodeTextItemizer) — the
    /// [`TextItemizer`](crate::text::shaping::TextItemizer) that splits text into
    /// uniform bidi-level + script runs before per-item resolution and shaping.
    ///
    /// Install it with
    /// [`set_text_itemizer`](crate::runtime::UiRuntime::set_text_itemizer). The
    /// itemizer is returned boxed as the domain port, so callers depend only on
    /// the port, not the concrete adapter.
    #[cfg(feature = "text-unicode")]
    #[must_use]
    pub fn text_itemizer(&self) -> Box<dyn crate::text::shaping::TextItemizer> {
        Box::new(crate::text_unicode::UnicodeTextItemizer::new())
    }

    /// Injects the **whole real text stack** into `runtime` over the faces in
    /// `fonts`, and returns the scene font handle assigned to each (spec §5.10,
    /// §5.18; §2.7 adapters keep host reach explicit).
    ///
    /// This is the one call that replaces all three of the runtime's honest but
    /// inert text defaults at once, which until now every host re-derived by hand:
    ///
    /// * the [`MockTextShaper`](crate::text::shaping::MockTextShaper) → the real
    ///   [`HarfrustShaper`](crate::text_harfrust::HarfrustShaper), so glyphs carry
    ///   font-specific indices with kerning/ligatures/GPOS rather than Unicode
    ///   scalars;
    /// * the empty [`MockFontResolver`](crate::text::font::MockFontResolver) → a
    ///   [`FontRegistry`](crate::text_harfrust::FontRegistry) over `fonts`, so a
    ///   run resolves to a face that actually *covers* it;
    /// * the [`SingleRunItemizer`](crate::text::shaping::SingleRunItemizer) → the
    ///   real [`UnicodeTextItemizer`](crate::text_unicode::UnicodeTextItemizer),
    ///   so text splits into uniform bidi-level + script runs — **the default has
    ///   no bidi at all**, and this is what gives it any.
    ///
    /// It also hands the shaper the registry's
    /// [`face_metrics`](crate::text_harfrust::FontRegistry::face_metrics) snapshot,
    /// taken after every face is registered. That is the step easiest to omit and
    /// hardest to notice: a shaper is asked for line metrics by
    /// [`FontId`](crate::text::font::FontId) alone, so without the table it shapes
    /// real glyphs into line boxes sized by idealized 0.8/0.2 ratios — real text in
    /// fake boxes.
    ///
    /// The facade constructs the concrete adapters and hands the runtime domain
    /// ports, so the application layer depends only on the ports, never on these
    /// adapter crates.
    ///
    /// # The host must still feed the renderer
    ///
    /// This installs the resolve/shape half. Rasterization lives in a renderer
    /// this kit does not own, and it must be given the *same* bytes under the
    /// *same* handles or it will paint the wrong outlines — see
    /// [`InstalledFonts`](super::InstalledFonts), which exists to make that
    /// hand-off one loop.
    ///
    /// # An empty `fonts` still renders — the bundled last-resort face (§5.18)
    ///
    /// The kit ships one vendored face, `assets/fonts/NotoSans-Regular.ttf`,
    /// embedded at compile time so it needs no filesystem — which the isolated /
    /// WASM target does not have. It is appended to the registry **after** the
    /// host's faces, under the generic `sans-serif` family every default fallback
    /// chain terminates in, so:
    ///
    /// * a host that supplies its own faces is unaffected — its faces match first
    ///   by family and coverage, and the bundled one only fills a gap nothing else
    ///   covers;
    /// * an **empty** `fonts` (the WASM/isolated case: nothing enumerated) falls
    ///   through to the bundled face and paints, rather than rendering nothing.
    ///
    /// This is §5.18's "bundled fallback set for WASM/isolated environments" met,
    /// not the deviation it once was. The bundled face is skipped when the host
    /// already provides those exact bytes, so it never double-registers.
    ///
    /// The returned [`InstalledFonts`](super::InstalledFonts) — the renderer
    /// hand-off table — lists the host's faces exactly, and additionally the
    /// bundled face **only when the host supplied none**: that is the one case the
    /// renderer must be handed the bundled bytes to paint from, while a host with
    /// its own faces keeps the handle table it already expects.
    #[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
    pub fn install_text_stack(
        &self,
        runtime: &mut crate::runtime::UiRuntime,
        fonts: &super::FontSet,
    ) -> super::InstalledFonts {
        use crate::text::font::{FontAttributes, FontFamily, GenericFamily};

        /// The vendored last-resort face, embedded so it needs no filesystem.
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

    /// Reads the host's own `sans-serif` / `serif` / `monospace` faces off disk via
    /// fontconfig, as a [`FontSet`](super::FontSet) ready for
    /// [`install_text_stack`](Self::install_text_stack) (spec §5.18).
    ///
    /// This is what "use the system font" has to mean for a toolkit that shapes its
    /// own text: a CSS family name is not enough — the shaper reads `cmap`/`GSUB`
    /// itself, so something must find the *file*. Each generic is registered under
    /// its own [`GenericFamily`](crate::text::font::GenericFamily), so
    /// `font-family: monospace` resolves to the host's mono face rather than to
    /// whatever came first.
    ///
    /// Returns `None` when fontconfig is unavailable or nominates nothing loadable.
    /// A caller that needs text regardless must supply bytes itself.
    ///
    /// # Scope
    ///
    /// **Linux only.** macOS (Core Text) and Windows (DirectWrite) are **not
    /// implemented** — this returns `None` there, and no text will paint from this
    /// path. Both want a real platform API rather than a CLI shell-out, and neither
    /// is built.
    ///
    /// # This is not for tests
    ///
    /// The faces it returns are whatever the machine happens to have installed, so
    /// any assertion about a metric drifts per machine. On the authoring machine
    /// `fc-match sans-serif` resolves into `/usr/share/fonts/WindowsFonts/`. Tests
    /// build a [`FontSet`](super::FontSet) from the vendored faces in
    /// `assets/fonts/` instead.
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
