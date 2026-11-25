//! [`ResourceBridge`]: the runtime's side of the §5.15–§5.19 resource tier.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::mpsc::{TryRecvError, channel};
use std::time::{Duration, Instant};

use obscur_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSink, DiagnosticSubsystem,
    RecoveryAction,
};
use obscur_geometry::{Rect, Size};
use obscur_identity::{NodeId, WindowId};
use obscur_layout::model::{BoxSizing, Dimension};
use obscur_resources::cache::{
    CacheBudget, CacheKey, CacheStatistics, EvictionPolicy, VisibilityPriority,
};
use obscur_resources::handle::{ResourceHandle, ResourceKind};
use obscur_resources::hub::{ResourceHub, ResourceRequest};
use obscur_resources::port::{ResourceDecoder, ResourceLoader, VectorDecoder};
use obscur_resources::source::{ResourceSource, dpi_variants};
#[cfg(test)]
use obscur_resources::state::ResourceState;
use obscur_resources::state::{ResourceError, ResourceMetadata, ResourceVersion};
use obscur_scene::paint::{SceneImage, SceneVector};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{BackgroundLayer, PropertyId};
use obscur_task::BlockingExecutor;
use obscur_tree::tree::RetainedTree;

use super::{
    ImageCache, ResourceOutcome, ResourceReporting, bitmap_info::BitmapInfo,
    deferred_load::DeferredLoad, pending_load::PendingLoad,
};
use crate::orchestration::{ComputedStyles, LayoutBoxes, LayoutStyles};

#[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/ResourceBridge.md")]
pub(crate) struct ResourceBridge {
    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/loader.md")]
    loader: Option<Arc<dyn ResourceLoader + Send + Sync>>,
    /// The injected decoder that turns a loaded image's encoded bytes into RGBA8
    /// pixels + intrinsic size (spec §5.16), or [`None`] when none was installed.
    /// Both a loader and a decoder are required to resolve an image; either absent
    /// means `url()` backgrounds paint nothing.
    decoder: Option<Arc<dyn ResourceDecoder + Send + Sync>>,
    /// The pure lifecycle/versioning registry for requested image resources (spec
    /// §5.15). It stores only lifecycle + metadata, never the decoded pixels — the
    /// pixels live in [`image_cache`](Self::image_cache) — so the two are kept in
    /// step by URL.
    hub: ResourceHub,
    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/image_cache.md")]
    image_cache: ImageCache,
    image_visibility: HashMap<Box<str>, VisibilityPriority>,
    known_images: HashMap<Box<str>, BitmapInfo>,
    cache_pressure_reported: bool,
    /// The host-installed port that parses vector sources (spec §5.17). Absent by
    /// default, in which case an SVG source paints nothing — the same policy an
    /// absent raster decoder has for a bitmap.
    vector_decoder: Option<Arc<dyn VectorDecoder + Send + Sync>>,
    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/vector_cache.md")]
    vector_cache: HashMap<Box<str>, SceneVector>,
    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/blocking_executor.md")]
    blocking_executor: Option<Arc<dyn BlockingExecutor + Send + Sync>>,
    /// Dispatched loads awaiting completion, cancellation or timeout.
    pending_loads: Vec<PendingLoad>,
    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/image_handles.md")]
    image_handles: HashMap<Box<str>, ResourceHandle>,
    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/image_referrers.md")]
    image_referrers: HashMap<Box<str>, NodeId>,
    /// Image sources this runtime has already reported as unresolvable because no
    /// loader/decoder is installed (spec §5.15 "The kit does not perform arbitrary
    /// network access by itself."). Deduplicates the diagnostic so a referenced
    /// URL is named once, not once per frame.
    unresolved_reported: HashSet<Box<str>>,
    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/deferred_loads.md")]
    deferred_loads: Vec<DeferredLoad>,
}

impl ResourceBridge {
    /// A bridge with no ports installed: the honest-capability default, in which
    /// every `url()` and every element `src` resolves to nothing and paints
    /// nothing.
    pub(crate) fn new() -> Self {
        Self {
            loader: None,
            decoder: None,
            hub: ResourceHub::new(),
            image_cache: ImageCache::new(),
            image_visibility: HashMap::new(),
            known_images: HashMap::new(),
            cache_pressure_reported: false,
            vector_decoder: None,
            vector_cache: HashMap::new(),
            blocking_executor: None,
            pending_loads: Vec::new(),
            image_handles: HashMap::new(),
            image_referrers: HashMap::new(),
            unresolved_reported: HashSet::new(),
            deferred_loads: Vec::new(),
        }
    }

    /// The decoded images, for the scene coordinator to paint from.
    pub(crate) fn images(&self) -> &HashMap<Box<str>, SceneImage> {
        self.image_cache.images()
    }

    /// The parsed vector documents, for the scene coordinator to paint from.
    pub(crate) const fn vectors(&self) -> &HashMap<Box<str>, SceneVector> {
        &self.vector_cache
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/state_of.md")]
    #[cfg(test)]
    pub(crate) fn state_of(&self, url: &str) -> Option<&ResourceState> {
        self.hub.state(*self.image_handles.get(url)?)
    }

    /// Replaces the shared source loader and raster decoder. Clears both caches and
    /// releases their handles so raster and vector URLs resolve against the new loader.
    pub(crate) fn set_providers(
        &mut self,
        loader: Arc<dyn ResourceLoader + Send + Sync>,
        decoder: Arc<dyn ResourceDecoder + Send + Sync>,
    ) {
        self.loader = Some(loader);
        self.decoder = Some(decoder);
        self.image_cache.clear();
        self.known_images.clear();
        self.cache_pressure_reported = false;
        self.vector_cache.clear();
        for (_, handle) in self.image_handles.drain() {
            self.hub.release(handle);
        }
        self.pending_loads.clear();
        self.image_referrers.clear();
        // Any load queued against the previous providers is abandoned — the new
        // pair re-resolves every URL from scratch, and a stale closure still holds
        // the old loader.
        self.deferred_loads.clear();
        // A loader is now installed, so URLs previously reported as unresolvable
        // may resolve; forget the reports so they are attempted afresh.
        self.unresolved_reported.clear();
    }

    /// Installs the blocking-work facility resource loads run on (spec §3.3).
    pub(crate) fn set_blocking_executor(
        &mut self,
        executor: Arc<dyn BlockingExecutor + Send + Sync>,
    ) {
        self.blocking_executor = Some(executor);
    }

    /// Replaces the vector decoder, clears its cache and releases vector requests made
    /// with the previous decoder.
    pub(crate) fn set_vector_decoder(&mut self, decoder: Arc<dyn VectorDecoder + Send + Sync>) {
        self.vector_decoder = Some(decoder);
        self.vector_cache.clear();
        self.image_handles.retain(|_, handle| {
            if handle.kind() == ResourceKind::Svg {
                self.hub.release(*handle);
                false
            } else {
                true
            }
        });
        self.pending_loads
            .retain(|load| load.handle.kind() != ResourceKind::Svg);
        self.deferred_loads
            .retain(|load| load.handle.kind() != ResourceKind::Svg);
        self.image_referrers.retain(|url, _| !is_vector_source(url));
        self.unresolved_reported
            .retain(|url| !is_vector_source(url));
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/run_deferred_loads.md")]
    pub(crate) fn run_deferred_loads(&mut self, report: &ResourceReporting<'_>) -> usize {
        let mut applied = 0usize;
        // Take the queue up front: `apply_outcome` borrows `self`, and a fresh
        // enqueue cannot happen here (no frame runs during a drain).
        for job in std::mem::take(&mut self.deferred_loads) {
            if self.apply_outcome((job.run)(), report) {
                applied += 1;
            }
        }
        applied
    }

    /// Applies finished off-thread loads and fails jobs whose executor dropped them or
    /// whose result is still pending after 60 seconds. Receivers for timed-out jobs
    /// are dropped, so a later result cannot overwrite the terminal failure. Returns
    /// the number of successful commits without requesting a frame.
    pub(crate) fn commit_results(&mut self, report: &ResourceReporting<'_>) -> usize {
        const LOAD_TIMEOUT: Duration = Duration::from_secs(60);
        let mut applied = 0;
        let now = Instant::now();
        for index in (0..self.pending_loads.len()).rev() {
            let pending = &self.pending_loads[index];
            let outcome = match pending.result.try_recv() {
                Ok(outcome) => outcome,
                Err(TryRecvError::Empty)
                    if now.duration_since(pending.submitted) < LOAD_TIMEOUT =>
                {
                    continue;
                }
                Err(error) => {
                    let error = match error {
                        TryRecvError::Disconnected => ResourceError::Cancelled,
                        TryRecvError::Empty => {
                            ResourceError::SourceUnavailable("resource load timed out".into())
                        }
                    };
                    ResourceOutcome::failed(
                        pending.key.clone(),
                        pending.handle,
                        pending.version,
                        error,
                    )
                }
            };
            self.pending_loads.swap_remove(index);
            if self.apply_outcome(outcome, report) {
                applied += 1;
            }
        }
        applied
    }

    /// Applies one finished load, returning whether it was accepted (a stale or
    /// failed job is not).
    fn apply_outcome(&mut self, outcome: ResourceOutcome, report: &ResourceReporting<'_>) -> bool {
        match outcome {
            ResourceOutcome::Image {
                key,
                handle,
                version,
                result: Ok(image),
            } => {
                // `intrinsic_size` is the pixel count and `scale` the density
                // of *this* representation, so a consumer derives the logical
                // size as one over the other (spec §5.16 DPI variants).
                let metadata = ResourceMetadata::new()
                    .with_intrinsic_size(intrinsic_of(&image))
                    .with_scale(image.scale());
                // The hub owns the staleness verdict; `false` means this job
                // was superseded, so its pixels must not reach the cache.
                if !self.hub.complete(handle, version, metadata) {
                    return false;
                }
                let priority = self.image_priority(&key);
                let retry = self
                    .known_images
                    .insert(
                        key.clone(),
                        BitmapInfo {
                            bytes: image.pixels().len() as u64,
                            logical_size: image.logical_size(),
                        },
                    )
                    .is_some();
                let evicted = self.image_cache.insert_for_frame(
                    key.clone(),
                    CacheKey::for_resource(handle.id()),
                    version,
                    image,
                    priority,
                    retry,
                );
                let rejected = !self.image_cache.images().contains_key(&key);
                let displaced = evicted
                    .iter()
                    .any(|url| self.image_visibility.contains_key(url));
                if rejected || displaced {
                    self.report_cache_pressure(report);
                }
                self.release_evicted(evicted);
                if rejected {
                    self.image_handles.remove(&key);
                    self.hub.release(handle);
                }
                true
            }
            ResourceOutcome::Vector {
                key,
                handle,
                version,
                result: Ok(vector),
            } => {
                let metadata = ResourceMetadata::new().with_intrinsic_size(vector.size());
                if !self.hub.complete(handle, version, metadata) {
                    return false;
                }
                self.vector_cache.insert(key, vector);
                true
            }
            ResourceOutcome::Image {
                key,
                handle,
                version,
                result: Err(error),
            }
            | ResourceOutcome::Vector {
                key,
                handle,
                version,
                result: Err(error),
            } => {
                // Only report failures the hub accepts for the current load.
                if self.hub.fail(handle, version, error.clone()) {
                    self.report_failure(handle, &key, &error, report);
                }
                false
            }
        }
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/release_evicted.md")]
    fn release_evicted(&mut self, evicted: Vec<Box<str>>) {
        for url in evicted {
            if let Some(handle) = self.image_handles.remove(&url) {
                self.hub.release(handle);
            }
        }
    }

    /// Sets the memory budget the decoded-image cache is held under, evicting down
    /// to it immediately, and returns how many images that evicted (spec §5.19
    /// *memory budgets*).
    pub(crate) fn set_image_cache_budget(&mut self, budget: CacheBudget) -> usize {
        self.cache_pressure_reported = false;
        let evicted = self.image_cache.set_budget(budget);
        let count = evicted.len();
        self.release_evicted(evicted);
        count
    }

    /// The memory budget the decoded-image cache is held under (spec §5.19).
    pub(crate) fn image_cache_budget(&self) -> CacheBudget {
        self.image_cache.budget()
    }

    /// Installs the strategy the decoded-image cache evicts by (spec §5.19
    /// *eviction*), replacing [`EvictionPolicy::VisibilityAware`] (the default).
    pub(crate) fn set_image_eviction_policy(&mut self, policy: EvictionPolicy) {
        self.image_cache.set_policy(policy);
    }

    /// The decoded-image cache's live metrics: hits, misses, evictions,
    /// insertions, resident entries and resident bytes (spec §5.19 *`DevTools`
    /// metrics*).
    pub(crate) fn image_cache_statistics(&self) -> CacheStatistics {
        self.image_cache.statistics()
    }

    /// How many resources the hub has been asked for, and how many decoded
    /// images / parsed vectors are resident.
    pub(crate) fn resource_count(&self) -> usize {
        self.hub.len()
    }

    pub(crate) fn resource_version(
        &self,
        id: obscur_identity::ResourceId,
    ) -> Option<obscur_resources::state::ResourceVersion> {
        self.hub.version(id)
    }

    /// The resident entry counts of the two asset caches, `(images, vectors)`.
    pub(crate) fn cached_counts(&self) -> (usize, usize) {
        (self.image_cache.len(), self.vector_cache.len())
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/bookkeeping_counts.md")]
    #[cfg(test)]
    pub(crate) fn bookkeeping_counts(&self) -> (usize, usize) {
        (self.image_handles.len(), self.image_referrers.len())
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/invalidate_dpi_variants.md")]
    pub(crate) fn invalidate_dpi_variants(&mut self) {
        let resident: Vec<Box<str>> = self.image_cache.images().keys().cloned().collect();
        self.image_cache.clear();
        self.release_evicted(resident);
        self.known_images.clear();
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/mark_image_visibility.md")]
    pub(crate) fn mark_image_visibility(
        &mut self,
        computed: &ComputedStyles,
        tree: &RetainedTree,
        boxes: &LayoutBoxes,
        viewport: Rect,
    ) {
        let mut referenced: HashMap<Box<str>, VisibilityPriority> = HashMap::new();
        for (node, style) in computed {
            for url in background_urls(style) {
                let priority = image_priority(boxes, *node, viewport);
                referenced
                    .entry(url.into())
                    .and_modify(|best| *best = (*best).max(priority))
                    .or_insert(priority);
            }
        }
        let element_sources: Vec<(NodeId, Box<str>)> = tree
            .iter()
            .filter_map(|(node, n)| n.attributes().image_source().map(|s| (node, s.into())))
            .collect();
        for (node, src) in element_sources {
            let priority = image_priority(boxes, node, viewport);
            referenced
                .entry(src)
                .and_modify(|best| *best = (*best).max(priority))
                .or_insert(priority);
        }
        self.image_cache.mark_visibility(&referenced);
        self.retain_referenced(&referenced);
        self.image_visibility = referenced;
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/retain_referenced.md")]
    fn retain_referenced(&mut self, referenced: &HashMap<Box<str>, VisibilityPriority>) {
        // A parsed document is small and re-parsing one costs what its first parse
        // cost, so liveness alone decides it. Decoded pixels are megabytes and the
        // whole point of `VisibilityPriority::Hidden` is that they SURVIVE going
        // off screen — which is why the image cache is not swept here and its
        // §5.19 budget owns that call instead.
        self.vector_cache
            .retain(|url, _| referenced.contains_key(url));
        // Resident pixels hold their own bookkeeping open: a handle is what names
        // a cached image to the hub, and dropping it while the pixels are still
        // there would leave the cache unable to say what it holds. The budget's
        // entry cap is what bounds this arm — without one, "resident" is as
        // unbounded as the leak being closed.
        let live: HashSet<&str> = referenced
            .keys()
            .map(Box::as_ref)
            .chain(self.image_cache.images().keys().map(Box::as_ref))
            .chain(self.vector_cache.keys().map(Box::as_ref))
            .collect();
        self.known_images
            .retain(|url, _| live.contains(url.as_ref()));
        let dropped: Vec<Box<str>> = self
            .image_handles
            .keys()
            .filter(|url| !live.contains(url.as_ref()))
            .cloned()
            .collect();
        for url in dropped {
            if let Some(handle) = self.image_handles.remove(&url) {
                // Released, not merely forgotten: the hub is refcounted, and an
                // entry nothing refers to is what `release` exists to reclaim. A
                // load still in flight for this URL then finds its handle gone and
                // `complete` refuses its pixels, which is the right answer — the
                // frame that asked for them is over and nothing shows them.
                self.hub.release(handle);
            }
        }
        self.pending_loads
            .retain(|load| self.hub.state(load.handle).is_some());
        self.image_referrers
            .retain(|url, _| live.contains(url.as_ref()));
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/resolve_background_images.md")]
    pub(crate) fn resolve_background_images(
        &mut self,
        computed: &ComputedStyles,
        device_scale: f32,
        report: &ResourceReporting<'_>,
    ) {
        for (node, style) in computed {
            for url in background_urls(style) {
                self.image_referrers.entry(url.into()).or_insert(*node);
                let vector = is_vector_source(url);
                if (!vector && self.image_cache.record_lookup(url))
                    || self.image_handles.contains_key(url)
                    || (!vector && self.defer_bitmap_retry(url, report))
                {
                    continue;
                }
                let key: Box<str> = url.into();
                let source = ResourceSource::file(key.clone());
                let kind = if vector {
                    ResourceKind::Svg
                } else {
                    ResourceKind::Image
                };
                let handle = self.hub.request(ResourceRequest::new(source.clone(), kind));
                self.image_handles.insert(key.clone(), handle);
                let Some(version) = self.hub.begin_load(handle) else {
                    continue;
                };
                let has_decoder = if vector {
                    self.vector_decoder.is_some()
                } else {
                    self.decoder.is_some()
                };
                if self.loader.is_none() || !has_decoder {
                    self.hub.fail(
                        handle,
                        version,
                        ResourceError::SourceUnavailable(
                            "no host loader or decoder is installed to service this source".into(),
                        ),
                    );
                    self.report_unresolvable_source(url, report.sink);
                } else if vector {
                    self.load_vector(key, source, handle, version);
                } else {
                    self.load_image(key, handle, version, device_scale);
                }
            }
        }
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/resolve_element_images.md")]
    pub(crate) fn resolve_element_images(
        &mut self,
        tree: &RetainedTree,
        device_scale: f32,
        report: &ResourceReporting<'_>,
    ) {
        // Both a loader and a decoder are required; without either, element images
        // honestly resolve to nothing — but each referenced `src` is named once on
        // the diagnostics channel so a missing provider is loud, not silent: the
        // kit does not fetch it by itself (spec §5.15).
        if self.loader.is_none() || self.decoder.is_none() {
            let sources: Vec<Box<str>> = tree
                .iter()
                .filter_map(|(_, node)| node.attributes().image_source().map(Box::<str>::from))
                .filter(|src| !is_vector_source(src))
                .collect();
            for src in sources {
                self.report_unresolvable_source(&src, report.sink);
            }
            return;
        }
        // Gather each element node's own image source up front, so the retained-tree
        // borrow is released before the hub / caches are mutated below. The node is
        // kept alongside the source so a later load failure can be attributed to the
        // element that referenced it (spec §1.6 node context).
        let sources: Vec<(NodeId, Box<str>)> = tree
            .iter()
            .filter_map(|(node, n)| {
                n.attributes()
                    .image_source()
                    .map(|src| (node, Box::<str>::from(src)))
            })
            // A vector source is not a bitmap: it goes through
            // `resolve_element_vectors`, and handing it to the raster decoder
            // would only produce a decode failure.
            .filter(|(_, src)| !is_vector_source(src))
            .collect();
        for (node, key) in sources {
            self.image_referrers.entry(key.clone()).or_insert(node);
            if self.image_cache.record_lookup(&key)
                || self.image_handles.contains_key(&key)
                || self.defer_bitmap_retry(&key, report)
            {
                // Already resolved, or already attempted (and failed): do not retry.
                continue;
            }
            let source = ResourceSource::file(key.clone());
            let handle = self
                .hub
                .request(ResourceRequest::new(source.clone(), ResourceKind::Image));
            self.image_handles.insert(key.clone(), handle);
            let Some(version) = self.hub.begin_load(handle) else {
                continue;
            };
            self.load_image(key, handle, version, device_scale);
        }
    }

    /// Resolves every not-yet-parsed element `src` that names a vector document
    /// into the parsed-document cache (spec §5.17).
    pub(crate) fn resolve_element_vectors(
        &mut self,
        tree: &RetainedTree,
        report: &ResourceReporting<'_>,
    ) {
        let has_providers = self.loader.is_some() && self.vector_decoder.is_some();
        let sources = tree.iter().filter_map(|(node, element)| {
            let src = element.attributes().image_source()?;
            is_vector_source(src).then_some((node, src))
        });
        for (node, src) in sources {
            if !has_providers {
                self.report_unresolvable_source(src, report.sink);
                continue;
            }
            let key = Box::<str>::from(src);
            self.image_referrers.entry(key.clone()).or_insert(node);
            if self.vector_cache.contains_key(&key) || self.image_handles.contains_key(&key) {
                // Parsed once. A theme change re-resolves `currentColor` at compose
                // time and must NOT re-parse (spec §5.17) — so a cached document is
                // never revisited here.
                continue;
            }
            let source = ResourceSource::file(key.clone());
            let handle = self
                .hub
                .request(ResourceRequest::new(source.clone(), ResourceKind::Svg));
            self.image_handles.insert(key.clone(), handle);
            let Some(version) = self.hub.begin_load(handle) else {
                continue;
            };
            self.load_vector(key, source, handle, version);
        }
    }

    /// Returns the strongest visibility of the URL across its current references.
    fn image_priority(&self, url: &str) -> VisibilityPriority {
        self.image_visibility
            .get(url)
            .copied()
            .unwrap_or(VisibilityPriority::Offscreen)
    }

    /// Defers a known bitmap when no room can be made without displacing equally or more visible images.
    fn defer_bitmap_retry(&mut self, url: &str, report: &ResourceReporting<'_>) -> bool {
        let deferred = self.known_images.get(url).is_some_and(|image| {
            !self
                .image_cache
                .can_retry(image.bytes, self.image_priority(url))
        });
        if deferred {
            self.report_cache_pressure(report);
        }
        deferred
    }

    /// Reports budget pressure once for the active provider and budget configuration.
    fn report_cache_pressure(&mut self, report: &ResourceReporting<'_>) {
        if self.cache_pressure_reported {
            return;
        }
        self.cache_pressure_reported = true;
        report.sink.emit(
            &Diagnostic::new(
                DiagnosticCode::new("resource.image_cache_pressure"),
                DiagnosticSeverity::Warning,
                DiagnosticSubsystem::Resources,
                "the image cache budget limited decoded pixels for referenced images",
            )
            .with_recovery(RecoveryAction::Fallback),
        );
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/load_image.md")]
    fn load_image(
        &mut self,
        key: Box<str>,
        handle: ResourceHandle,
        version: ResourceVersion,
        device_scale: f32,
    ) {
        let (Some(loader), Some(decoder)) = (self.loader.clone(), self.decoder.clone()) else {
            return;
        };
        // The surface's density decides which variants are worth asking for: at
        // `1.0` this is just the authored path, so the common case is unchanged.
        let candidates = dpi_variants(&key, device_scale);
        self.submit_load(key, handle, version, move |key| {
            ResourceOutcome::load_image(&*loader, &*decoder, &candidates, key, handle, version)
        });
    }

    /// Runs one vector load — off the frame thread when a blocking executor is
    /// installed, on it when none is. The image counterpart of
    /// [`load_image`](Self::load_image).
    fn load_vector(
        &mut self,
        key: Box<str>,
        source: ResourceSource,
        handle: ResourceHandle,
        version: ResourceVersion,
    ) {
        let (Some(loader), Some(decoder)) = (self.loader.clone(), self.vector_decoder.clone())
        else {
            return;
        };
        self.submit_load(key, handle, version, move |key| {
            ResourceOutcome::load_vector(&*loader, &*decoder, &source, key, handle, version)
        });
    }

    /// Runs a load outside the frame phase and converts provider panics into failed outcomes.
    fn submit_load(
        &mut self,
        key: Box<str>,
        handle: ResourceHandle,
        version: ResourceVersion,
        load: impl FnOnce(Box<str>) -> ResourceOutcome + Send + 'static,
    ) {
        let job_key = key.clone();
        let job = move || {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| load(job_key.clone())))
                .unwrap_or_else(|panic| {
                    // A custom panic payload can itself panic on drop.
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(panic)))
                        .map_err(std::mem::forget);
                    ResourceOutcome::failed(
                        job_key,
                        handle,
                        version,
                        ResourceError::DecodeFailed("resource loader or decoder panicked".into()),
                    )
                })
        };
        if let Some(executor) = self.blocking_executor.clone() {
            let (sender, result) = channel();
            self.pending_loads.push(PendingLoad {
                key,
                handle,
                version,
                submitted: Instant::now(),
                result,
            });
            executor.run_blocking(Box::new(move || {
                let _ = sender.send(job());
            }));
        } else {
            self.deferred_loads.push(DeferredLoad {
                handle,
                run: Box::new(job),
            });
        }
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/size_replaced_elements.md")]
    pub(crate) fn size_replaced_elements(
        &self,
        tree: &RetainedTree,
        layout_styles: &mut LayoutStyles,
    ) {
        // Collect (node, logical size) first so the retained-tree borrow is
        // released before the styles map is mutated.
        let intrinsic: Vec<(NodeId, (f32, f32))> = tree
            .iter()
            .filter_map(|(node, n)| {
                let src = n.attributes().image_source()?;
                let size = self
                    .known_images
                    .get(src)
                    .map(|info| info.logical_size)
                    .or_else(|| {
                        self.vector_cache
                            .get(src)
                            .map(|vector| (vector.size().width, vector.size().height))
                    })?;
                Some((node, size))
            })
            .collect();
        for (node, (iw, ih)) in intrinsic {
            if iw <= 0.0 || ih <= 0.0 {
                continue;
            }
            let Some(style) = layout_styles.get_mut(&node) else {
                continue;
            };
            let auto_width = style.width.is_auto();
            let auto_height = style.height.is_auto();
            let contain_width = style.size_containment.contains_width(style.writing_mode);
            let contain_height = style.size_containment.contains_height(style.writing_mode);
            let border_box = style.box_sizing == BoxSizing::BorderBox;
            if contain_width && auto_width {
                let insets = style.padding.left
                    + style.padding.right
                    + style.border.left
                    + style.border.right;
                let intrinsic = style.contained_intrinsic_size.width;
                style.width = Dimension::Px(intrinsic + if border_box { insets } else { 0.0 });
            } else if auto_width && auto_height {
                style.width = Dimension::Px(iw);
            }
            if contain_height && auto_height {
                let insets = style.padding.top
                    + style.padding.bottom
                    + style.border.top
                    + style.border.bottom;
                let intrinsic = style.contained_intrinsic_size.height;
                style.height = Dimension::Px(intrinsic + if border_box { insets } else { 0.0 });
            } else if auto_width && auto_height {
                style.height = Dimension::Px(ih);
            }
        }
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/report_unresolvable_source.md")]
    fn report_unresolvable_source(&mut self, key: &str, sink: &dyn DiagnosticSink) {
        if !self.unresolved_reported.insert(key.into()) {
            return;
        }
        sink.emit(&Diagnostic::new(
            DiagnosticCode::new("resource.no_loader"),
            DiagnosticSeverity::Error,
            DiagnosticSubsystem::Resources,
            format!(
                "no resource loader/decoder is installed to resolve `{key}`; the \
                 kit does not perform network access by itself (spec §5.15), so \
                 the reference is left unresolved and paints nothing"
            ),
        ));
    }

    #[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/report_failure.md")]
    fn report_failure(
        &self,
        handle: ResourceHandle,
        key: &str,
        error: &ResourceError,
        report: &ResourceReporting<'_>,
    ) {
        const CODE: &str = "resource.load_failed";
        let mut diagnostic = Diagnostic::new(
            DiagnosticCode::new(CODE),
            DiagnosticSeverity::Error,
            DiagnosticSubsystem::Resources,
            localized_diagnostic_message(CODE, report.locale.language(), key, error),
        );
        diagnostic.context.resource = Some(handle.id());
        // Attributed to the node that referenced the resource (recorded at request
        // time in the resolve passes) and to this runtime's window surface — this
        // runtime is one window (spec §1.6 node/resource/window context).
        diagnostic.context.node = self.image_referrers.get(key).copied();
        diagnostic.context.window = Some(WindowId::from_raw(report.runtime.raw()));
        // The frame is not torn down for a missing image: the reference resolves
        // to nothing and the box paints empty, keeping the layout around it intact
        // (spec §1.6 "fallback and recovery are preferred where correctness can be
        // maintained"; §5.15 failed state). That substitution is the recovery.
        diagnostic.recovery = RecoveryAction::Fallback;
        report.sink.emit(&diagnostic);
    }
}

impl std::fmt::Debug for ResourceBridge {
    /// Names what is installed and how much is resident, never the assets
    /// themselves: a decoded image is megabytes of pixels and a `dyn` port has no
    /// useful rendering.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceBridge")
            .field("loader", &self.loader.is_some())
            .field("decoder", &self.decoder.is_some())
            .field("vector_decoder", &self.vector_decoder.is_some())
            .field("blocking_executor", &self.blocking_executor.is_some())
            .field("images", &self.image_cache.len())
            .field("vectors", &self.vector_cache.len())
            .field("deferred_loads", &self.deferred_loads.len())
            .finish_non_exhaustive()
    }
}

/// `node`'s contribution to an image's visibility: on screen when its arranged
/// border box intersects `viewport`. A node with no arranged box (never laid
/// out, or `display: none`) shows nothing, so it contributes `Offscreen`.
fn image_priority(boxes: &LayoutBoxes, node: NodeId, viewport: Rect) -> VisibilityPriority {
    match boxes.get(&node) {
        Some(layout_box) if rects_intersect(layout_box.bounds, viewport) => {
            VisibilityPriority::Visible
        }
        _ => VisibilityPriority::Offscreen,
    }
}

#[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/rects_intersect.md")]
fn rects_intersect(a: Rect, b: Rect) -> bool {
    a.origin.x < b.right()
        && b.origin.x < a.right()
        && a.origin.y < b.bottom()
        && b.origin.y < a.bottom()
}

/// A decoded image's intrinsic size — its pixel dimensions, which is what
/// `object-fit` scales from (spec §5.16).
#[allow(clippy::cast_precision_loss)]
fn intrinsic_of(image: &SceneImage) -> Size {
    Size::new(image.width() as f32, image.height() as f32)
}

#[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/localized_diagnostic_message.md")]
fn localized_diagnostic_message(
    code: &str,
    lang: &str,
    key: &str,
    error: &ResourceError,
) -> String {
    match code {
        "resource.load_failed" => match lang {
            // Russian — a second real language so the code can be proven
            // locale-invariant while the message localizes.
            "ru" => {
                format!("не удалось загрузить или декодировать ресурс «{key}»: {error:?}")
            }
            // English is the fallback for `und` and every other language.
            _ => format!("failed to load or decode image `{key}`: {error:?}"),
        },
        // A class with no localized entry keeps its English wording.
        _ => format!("failed to load or decode image `{key}`: {error:?}"),
    }
}

/// Iterates resource URLs in single-layer and comma-separated backgrounds.
fn background_urls(style: &ComputedStyle) -> impl Iterator<Item = &str> {
    let value = style.get(PropertyId::BackgroundImage);
    value
        .and_then(ComputedValue::as_background_image_url)
        .into_iter()
        .chain(
            value
                .and_then(ComputedValue::as_background_layers)
                .into_iter()
                .flatten()
                .filter_map(|layer| match layer {
                    BackgroundLayer::Url(url) => Some(url.as_ref()),
                    _ => None,
                }),
        )
}

#[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/is_vector_source.md")]
fn is_vector_source(src: &str) -> bool {
    // Compared case-insensitively on the extension alone rather than by
    // lowercasing the whole path and matching a suffix: same answer, no
    // allocation, and it runs once per element source per frame.
    let extension = match src.rsplit_once('.') {
        Some((_, extension)) => extension,
        None => return false,
    };
    extension.eq_ignore_ascii_case("svg") || extension.eq_ignore_ascii_case("svgz")
}

#[cfg(test)]
mod tests {
    use super::{
        ResourceBridge, ResourceReporting, is_vector_source, localized_diagnostic_message,
        rects_intersect,
    };
    use obscur_diagnostics::NullDiagnosticSink;
    use obscur_geometry::Point;
    use obscur_geometry::{Rect, Size};
    use obscur_identity::{ComponentId, NodeId, RuntimeId};
    use obscur_localization::locale::LanguageTag;
    use obscur_resources::cache::CacheBudget;
    use obscur_resources::handle::ResourceKind;
    use obscur_resources::port::{
        DecodedResource, LoadedBytes, ResourceDecoder, ResourceLoader, VectorDecoder,
    };
    use obscur_resources::source::ResourceSource;
    use obscur_resources::state::{ResourceError, ResourceMetadata};
    use obscur_scene::paint::SceneVector;
    use obscur_tree::node::NodeKind;
    use obscur_tree::tree::RetainedTree;
    use obscur_view::view::{SRC, StyleAttributes, ViewProperty, ViewValue};
    use std::sync::Arc;

    use crate::orchestration::{ComputedStyles, LayoutBoxes};

    /// A loader that answers every source with the same seven bytes, or refuses
    /// every one of them — the two states a URL's bookkeeping has to survive.
    struct Loader {
        succeeds: bool,
    }

    impl ResourceLoader for Loader {
        fn can_load(&self, _source: &ResourceSource) -> bool {
            self.succeeds
        }

        fn load(&self, _source: &ResourceSource) -> Result<LoadedBytes, ResourceError> {
            if self.succeeds {
                Ok(LoadedBytes::new(
                    Arc::from(&b"encoded"[..]),
                    ResourceMetadata::new(),
                ))
            } else {
                Err(ResourceError::NotFound)
            }
        }
    }

    /// One opaque pixel, whatever it was handed.
    struct OnePixel;

    impl ResourceDecoder for OnePixel {
        fn supports(&self, kind: ResourceKind) -> bool {
            kind == ResourceKind::Image
        }

        fn decode(
            &self,
            _kind: ResourceKind,
            _bytes: &[u8],
        ) -> Result<DecodedResource, ResourceError> {
            Ok(DecodedResource::new(
                Arc::from(&[0xFF, 0x00, 0x00, 0xFF][..]),
                ResourceMetadata::new().with_intrinsic_size(Size::new(1.0, 1.0)),
            ))
        }
    }

    /// An empty 1x1 document, whatever it was handed.
    struct EmptyVector;

    impl VectorDecoder for EmptyVector {
        fn decode_vector(&self, _bytes: &[u8]) -> Result<SceneVector, ResourceError> {
            Ok(SceneVector::new(Size::new(1.0, 1.0), Vec::new()))
        }
    }

    #[derive(Default)]
    struct HoldingExecutor(std::sync::Mutex<Option<Box<dyn FnOnce() + Send>>>);

    impl obscur_task::BlockingExecutor for HoldingExecutor {
        fn run_blocking(&self, job: Box<dyn FnOnce() + Send>) {
            *self.0.lock().unwrap() = Some(job);
        }
    }

    /// Verifies cancellation, expiry, and rejection of a completion that arrives after expiry.
    #[test]
    fn discarded_and_expired_loads_fail() {
        use obscur_diagnostics::{Diagnostic, DiagnosticSink};
        use std::cell::RefCell;
        use std::time::Duration;

        #[derive(Default)]
        struct Recorder(RefCell<Vec<Diagnostic>>);

        impl DiagnosticSink for Recorder {
            fn emit(&self, diagnostic: &Diagnostic) {
                self.0.borrow_mut().push(diagnostic.clone());
            }
        }

        let locale = LanguageTag::parse(LOCALE).unwrap();
        for expired in [false, true] {
            let sink = Recorder::default();
            let report = ResourceReporting {
                sink: &sink,
                locale: &locale,
                runtime: RuntimeId::from_raw(1),
            };
            let executor = Arc::new(HoldingExecutor::default());
            let mut bridge = ResourceBridge::new();
            bridge.set_providers(Arc::new(Loader { succeeds: true }), Arc::new(OnePixel));
            bridge.set_blocking_executor(executor.clone());
            let mut tree = rooted();
            mount(&mut tree, "icon.png");
            bridge.resolve_element_images(&tree, 1.0, &report);
            assert!(bridge.state_of("icon.png").unwrap().is_loading());
            assert_eq!(bridge.commit_results(&report), 0);

            let job = executor.0.lock().unwrap().take().unwrap();
            let late_job = if expired {
                bridge.pending_loads[0].submitted -= Duration::from_secs(61);
                Some(job)
            } else {
                drop(job);
                None
            };
            assert_eq!(bridge.commit_results(&report), 0);
            assert!(bridge.state_of("icon.png").unwrap().is_failed());
            assert!(bridge.pending_loads.is_empty());
            if let Some(job) = late_job {
                job();
                assert_eq!(bridge.commit_results(&report), 0);
                assert!(bridge.state_of("icon.png").unwrap().is_failed());
                assert_eq!(bridge.cached_counts(), (0, 0));
            }
            let diagnostics = sink.0.borrow();
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].code.as_str(), "resource.load_failed");
            assert!(diagnostics[0].message.contains("icon.png"));
        }
    }

    /// Checks provider replacement across queued raster and vector loads and late off-thread completions.
    #[test]
    fn replacing_providers_releases_handles_and_cancels_old_loads() {
        let sink = NullDiagnosticSink;
        let locale = LanguageTag::parse(LOCALE).unwrap();
        let report = reporting(&sink, &locale);
        let mut bridge = ResourceBridge::new();
        bridge.set_providers(Arc::new(Loader { succeeds: true }), Arc::new(OnePixel));
        bridge.set_vector_decoder(Arc::new(EmptyVector));

        let mut tree = rooted();
        mount(&mut tree, "icon.png");
        mount(&mut tree, "icon.svg");
        bridge.resolve_element_images(&tree, 1.0, &report);
        bridge.resolve_element_vectors(&tree, &report);
        assert_eq!(bridge.resource_count(), 2);

        bridge.set_vector_decoder(Arc::new(EmptyVector));
        assert_eq!(bridge.resource_count(), 1);
        assert_eq!(bridge.run_deferred_loads(&report), 1);
        assert_eq!(bridge.cached_counts(), (1, 0));

        let executor = Arc::new(HoldingExecutor::default());
        bridge.set_blocking_executor(executor.clone());
        bridge.resolve_element_vectors(&tree, &report);
        let old_job = executor.0.lock().unwrap().take().unwrap();
        assert_eq!(bridge.resource_count(), 2);

        bridge.set_providers(Arc::new(Loader { succeeds: true }), Arc::new(OnePixel));
        assert_eq!(bridge.resource_count(), 0);
        assert_eq!(bridge.cached_counts(), (0, 0));

        bridge.resolve_element_vectors(&tree, &report);
        let new_job = executor.0.lock().unwrap().take().unwrap();
        old_job();
        assert_eq!(bridge.commit_results(&report), 0);
        assert!(bridge.state_of("icon.svg").unwrap().is_loading());
        assert_eq!(bridge.cached_counts(), (0, 0));

        new_job();
        assert_eq!(bridge.commit_results(&report), 1);
        assert_eq!(bridge.cached_counts(), (0, 1));
        assert_eq!(bridge.resource_count(), 1);
    }

    const LOCALE: &str = "en";

    fn reporting<'a>(
        sink: &'a NullDiagnosticSink,
        locale: &'a LanguageTag,
    ) -> ResourceReporting<'a> {
        ResourceReporting {
            sink,
            runtime: RuntimeId::from_raw(1),
            locale,
        }
    }

    fn rooted() -> RetainedTree {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        tree
    }

    fn mount(tree: &mut RetainedTree, src: &str) -> NodeId {
        let node = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(1),
        });
        let prop = ViewProperty::new(SRC, ViewValue::Text(src.into()));
        tree.get_mut(node)
            .expect("just allocated")
            .set_attributes(StyleAttributes::extract(&[prop]));
        node
    }

    fn sweep(bridge: &mut ResourceBridge, tree: &RetainedTree) {
        bridge.mark_image_visibility(
            &ComputedStyles::default(),
            tree,
            &LayoutBoxes::default(),
            Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0)),
        );
    }

    #[test]
    fn a_fresh_bridge_has_no_ports_and_nothing_resident() {
        let bridge = ResourceBridge::new();
        assert_eq!(bridge.cached_counts(), (0, 0));
        assert_eq!(bridge.resource_count(), 0);
        assert!(bridge.images().is_empty());
        assert!(bridge.vectors().is_empty());
    }

    #[test]
    fn a_vector_source_is_decided_by_extension_and_is_case_insensitive() {
        assert!(is_vector_source("icon.svg"));
        assert!(is_vector_source("ICON.SVGZ"));
        assert!(!is_vector_source("photo.png"));
        // Not a suffix match on the whole name: `svg` in the middle is a bitmap.
        assert!(!is_vector_source("svg-logo.png"));
    }

    #[test]
    fn a_touching_edge_is_not_an_intersection() {
        let a = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));
        // Shares the right edge exactly: no pixel is shown, so no intersection.
        let touching = Rect::new(Point::new(10.0, 0.0), Size::new(10.0, 10.0));
        let overlapping = Rect::new(Point::new(9.0, 0.0), Size::new(10.0, 10.0));
        assert!(!rects_intersect(a, touching));
        assert!(rects_intersect(a, overlapping));
        // A zero-extent box intersects nothing, including itself.
        let empty = Rect::new(Point::new(0.0, 0.0), Size::new(0.0, 0.0));
        assert!(!rects_intersect(empty, a));
    }

    #[test]
    fn the_message_localizes_while_the_code_does_not() {
        let error = ResourceError::NotFound;
        let en = localized_diagnostic_message("resource.load_failed", "en", "logo.png", &error);
        let ru = localized_diagnostic_message("resource.load_failed", "ru", "logo.png", &error);
        let und = localized_diagnostic_message("resource.load_failed", "und", "logo.png", &error);
        assert_ne!(en, ru, "the human wording follows the language");
        assert_eq!(en, und, "an unlocalized language falls back to English");
        // A fault class with no catalog entry keeps the English wording rather
        // than producing an empty or code-shaped message.
        let unknown = localized_diagnostic_message("resource.unknown", "ru", "logo.png", &error);
        assert_eq!(unknown, en);
    }

    #[test]
    fn an_unreferenced_url_gives_up_its_handle_and_its_referrer() {
        // Nothing resident to justify keeping them: the load fails, so the two
        // maps hold the URL and nothing else does. Unmount the element and they
        // must be back to the size they were before it was ever mounted — the
        // exact shape of the leak, one entry per URL ever seen.
        let sink = NullDiagnosticSink;
        let locale = LanguageTag::parse(LOCALE).expect("a real tag");
        let report = reporting(&sink, &locale);

        let mut bridge = ResourceBridge::new();
        bridge.set_providers(Arc::new(Loader { succeeds: false }), Arc::new(OnePixel));
        let mut tree = rooted();
        assert_eq!(bridge.bookkeeping_counts(), (0, 0), "prior size");

        let node = mount(&mut tree, "thumb.png");
        bridge.resolve_element_images(&tree, 1.0, &report);
        bridge.run_deferred_loads(&report);
        assert_eq!(
            bridge.bookkeeping_counts(),
            (1, 1),
            "requested and attributed"
        );
        assert_eq!(
            bridge.cached_counts().0,
            0,
            "the load failed, so nothing is resident"
        );

        tree.remove(node);
        sweep(&mut bridge, &tree);
        assert_eq!(
            bridge.bookkeeping_counts(),
            (0, 0),
            "back to the prior size"
        );
    }

    #[test]
    fn an_unreferenced_vector_leaves_the_cache_with_its_handle() {
        // The vector cache had no budget at all and was emptied only when the
        // decoder was replaced — which for a running program is never.
        let sink = NullDiagnosticSink;
        let locale = LanguageTag::parse(LOCALE).expect("a real tag");
        let report = reporting(&sink, &locale);

        let mut bridge = ResourceBridge::new();
        bridge.set_providers(Arc::new(Loader { succeeds: true }), Arc::new(OnePixel));
        bridge.set_vector_decoder(Arc::new(EmptyVector));
        let mut tree = rooted();

        let node = mount(&mut tree, "icon.svg");
        bridge.resolve_element_vectors(&tree, &report);
        bridge.run_deferred_loads(&report);
        assert_eq!(bridge.cached_counts().1, 1, "parsed once and cached");
        assert_eq!(bridge.bookkeeping_counts().0, 1);

        tree.remove(node);
        sweep(&mut bridge, &tree);
        assert_eq!(bridge.cached_counts().1, 0, "back to the prior size");
        assert_eq!(bridge.bookkeeping_counts(), (0, 0));
    }

    #[test]
    fn bookkeeping_cannot_outgrow_the_cache_that_justifies_it() {
        // A file manager walking a directory: a new thumbnail URL per file, each
        // shown once and then scrolled past. The decoded pixels are deliberately
        // kept — that is what the cache is for — so the bookkeeping is allowed to
        // be as large as the cache and no larger. Without the sweep it is one
        // entry per file EVER SEEN, and the cap below is the whole difference.
        let sink = NullDiagnosticSink;
        let locale = LanguageTag::parse(LOCALE).expect("a real tag");
        let report = reporting(&sink, &locale);

        let mut bridge = ResourceBridge::new();
        bridge.set_providers(Arc::new(Loader { succeeds: true }), Arc::new(OnePixel));
        bridge.set_image_cache_budget(CacheBudget::entries(2));
        let mut tree = rooted();

        for i in 0..40 {
            let node = mount(&mut tree, &format!("file-{i}.png"));
            bridge.resolve_element_images(&tree, 1.0, &report);
            bridge.run_deferred_loads(&report);
            sweep(&mut bridge, &tree);
            tree.remove(node);
            sweep(&mut bridge, &tree);
        }
        let (handles, referrers) = bridge.bookkeeping_counts();
        assert!(handles <= 2, "{handles} handles for a 2-entry cache");
        assert!(referrers <= 2, "{referrers} referrers for a 2-entry cache");
        assert_eq!(bridge.cached_counts().0, 2, "the budget held");
    }
}
