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
    /// the reference wiring of the frame pipeline for reproducible, offscreen
    /// rendering and tests. Always available (no feature gate).
    #[must_use]
    pub fn headless_host(&self) -> crate::headless::HeadlessHost {
        crate::headless::HeadlessHost::new(crate::identity::RuntimeId::from_raw(self.runtime_id))
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
    #[cfg(feature = "task-smol")]
    #[must_use]
    pub fn default_task_spawner(&self) -> crate::task_smol::spawner::SmolSpawner {
        crate::task_smol::spawner::SmolSpawner::new()
    }

    /// Builds the platform shell provider. On Linux this shells out to
    /// `xdg-open`; the dialog and notification providers are obtained the same
    /// way from `crate::system_platform::provider`.
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn system_shell_provider(&self) -> crate::system_platform::provider::LinuxShellProvider {
        crate::system_platform::provider::LinuxShellProvider::new()
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
        let _ = renderer.supports_effect("blur");
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
}
