#![doc = include_str!("resource_bridge.doc/_module.md")]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use obscur_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSink, DiagnosticSubsystem,
    RecoveryAction,
};
use obscur_geometry::{Rect, Size};
use obscur_identity::{NodeId, RuntimeId, WindowId};
use obscur_layout::model::Dimension;
use obscur_localization::locale::LanguageTag;
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
use obscur_style::computed::ComputedValue;
use obscur_style::property::PropertyId;
use obscur_task::{BlockingExecutor, TaskId, TaskResult, TaskScope};
use obscur_tree::tree::RetainedTree;

use crate::orchestration::{ComputedStyles, LayoutBoxes, LayoutStyles};
use crate::runtime::{ImageCache, ResourceOutcome};

#[doc = include_str!("resource_bridge.doc/ResourceBridge.md")]
pub(crate) struct ResourceBridge {
    #[doc = include_str!("resource_bridge.doc/loader.md")]
    loader: Option<Arc<dyn ResourceLoader + Send + Sync>>,
    #[doc = include_str!("resource_bridge.doc/decoder.md")]
    decoder: Option<Arc<dyn ResourceDecoder + Send + Sync>>,
    #[doc = include_str!("resource_bridge.doc/hub.md")]
    hub: ResourceHub,
    #[doc = include_str!("resource_bridge.doc/image_cache.md")]
    image_cache: ImageCache,
    #[doc = include_str!("resource_bridge.doc/vector_decoder.md")]
    vector_decoder: Option<Arc<dyn VectorDecoder + Send + Sync>>,
    #[doc = include_str!("resource_bridge.doc/vector_cache.md")]
    vector_cache: HashMap<Box<str>, SceneVector>,
    #[doc = include_str!("resource_bridge.doc/blocking_executor.md")]
    blocking_executor: Option<Arc<dyn BlockingExecutor + Send + Sync>>,
    #[doc = include_str!("resource_bridge.doc/tx.md")]
    tx: Sender<TaskResult<ResourceOutcome>>,
    #[doc = include_str!("resource_bridge.doc/rx.md")]
    rx: Receiver<TaskResult<ResourceOutcome>>,
    #[doc = include_str!("resource_bridge.doc/next_task.md")]
    next_task: u64,
    #[doc = include_str!("resource_bridge.doc/image_handles.md")]
    image_handles: HashMap<Box<str>, ResourceHandle>,
    #[doc = include_str!("resource_bridge.doc/image_referrers.md")]
    image_referrers: HashMap<Box<str>, NodeId>,
    #[doc = include_str!("resource_bridge.doc/unresolved_reported.md")]
    unresolved_reported: HashSet<Box<str>>,
    #[doc = include_str!("resource_bridge.doc/deferred_loads.md")]
    deferred_loads: Vec<Box<dyn FnOnce() -> ResourceOutcome + Send>>,
}

#[doc = include_str!("resource_bridge.doc/ResourceReporting.md")]
pub(crate) struct ResourceReporting<'a> {
    #[doc = include_str!("resource_bridge.doc/sink.md")]
    pub(crate) sink: &'a dyn DiagnosticSink,
    #[doc = include_str!("resource_bridge.doc/runtime.md")]
    pub(crate) runtime: RuntimeId,
    #[doc = include_str!("resource_bridge.doc/locale.md")]
    pub(crate) locale: &'a LanguageTag,
}

impl ResourceBridge {
    #[doc = include_str!("resource_bridge.doc/new.md")]
    pub(crate) fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            loader: None,
            decoder: None,
            hub: ResourceHub::new(),
            image_cache: ImageCache::new(),
            vector_decoder: None,
            vector_cache: HashMap::new(),
            blocking_executor: None,
            tx,
            rx,
            next_task: 0,
            image_handles: HashMap::new(),
            image_referrers: HashMap::new(),
            unresolved_reported: HashSet::new(),
            deferred_loads: Vec::new(),
        }
    }

    #[doc = include_str!("resource_bridge.doc/images.md")]
    pub(crate) fn images(&self) -> &HashMap<Box<str>, SceneImage> {
        self.image_cache.images()
    }

    #[doc = include_str!("resource_bridge.doc/vectors.md")]
    pub(crate) const fn vectors(&self) -> &HashMap<Box<str>, SceneVector> {
        &self.vector_cache
    }

    #[doc = include_str!("resource_bridge.doc/state_of.md")]
    #[cfg(test)]
    pub(crate) fn state_of(&self, url: &str) -> Option<&ResourceState> {
        self.hub.state(*self.image_handles.get(url)?)
    }

    #[doc = include_str!("resource_bridge.doc/set_providers.md")]
    pub(crate) fn set_providers(
        &mut self,
        loader: Arc<dyn ResourceLoader + Send + Sync>,
        decoder: Arc<dyn ResourceDecoder + Send + Sync>,
    ) {
        self.loader = Some(loader);
        self.decoder = Some(decoder);
        self.image_cache.clear();
        self.image_handles.clear();
        self.image_referrers.clear();
        // Any load queued against the previous providers is abandoned — the new
        // pair re-resolves every URL from scratch, and a stale closure still holds
        // the old loader.
        self.deferred_loads.clear();
        // A loader is now installed, so URLs previously reported as unresolvable
        // may resolve; forget the reports so they are attempted afresh.
        self.unresolved_reported.clear();
    }

    #[doc = include_str!("resource_bridge.doc/set_blocking_executor.md")]
    pub(crate) fn set_blocking_executor(
        &mut self,
        executor: Arc<dyn BlockingExecutor + Send + Sync>,
    ) {
        self.blocking_executor = Some(executor);
    }

    #[doc = include_str!("resource_bridge.doc/set_vector_decoder.md")]
    pub(crate) fn set_vector_decoder(&mut self, decoder: Arc<dyn VectorDecoder + Send + Sync>) {
        self.vector_decoder = Some(decoder);
        self.vector_cache.clear();
        self.image_handles.clear();
    }

    #[doc = include_str!("resource_bridge.doc/run_deferred_loads.md")]
    pub(crate) fn run_deferred_loads(&mut self, report: &ResourceReporting<'_>) -> usize {
        let mut applied = 0usize;
        // Take the queue up front: `apply_outcome` borrows `self`, and a fresh
        // enqueue cannot happen here (no frame runs during a drain).
        for job in std::mem::take(&mut self.deferred_loads) {
            if self.apply_outcome(job(), report) {
                applied += 1;
            }
        }
        applied
    }

    #[doc = include_str!("resource_bridge.doc/commit_results.md")]
    pub(crate) fn commit_results(&mut self, report: &ResourceReporting<'_>) -> usize {
        let mut applied = 0usize;
        while let Ok(message) = self.rx.try_recv() {
            if self.apply_outcome(message.into_payload(), report) {
                applied += 1;
            }
        }
        applied
    }

    #[doc = include_str!("resource_bridge.doc/apply_outcome.md")]
    fn apply_outcome(&mut self, outcome: ResourceOutcome, report: &ResourceReporting<'_>) -> bool {
        match outcome {
            ResourceOutcome::Image {
                key,
                handle,
                version,
                result,
            } => match result {
                Ok(image) => {
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
                    // The pixels enter the cache through its §5.19 budget: an
                    // over-budget cache evicts to seat them, and the URLs it gave up
                    // must lose their hub bookkeeping or they could never load again.
                    let evicted = self.image_cache.insert(
                        key,
                        CacheKey::for_resource(handle.id()),
                        version,
                        image,
                    );
                    self.release_evicted(evicted);
                    true
                }
                Err(error) => {
                    // The hub rejects a stale/superseded failure; report only a
                    // fresh one, so a resource fails loudly exactly once (with its
                    // identity attached) rather than on every late arrival.
                    if self.hub.fail(handle, version, error.clone()) {
                        self.report_failure(handle, &key, &error, report);
                    }
                    false
                }
            },
            ResourceOutcome::Vector {
                key,
                handle,
                version,
                result,
            } => match result {
                Ok(vector) => {
                    let metadata = ResourceMetadata::new().with_intrinsic_size(vector.size());
                    if !self.hub.complete(handle, version, metadata) {
                        return false;
                    }
                    self.vector_cache.insert(key, vector);
                    true
                }
                Err(error) => {
                    self.hub.fail(handle, version, error);
                    false
                }
            },
        }
    }

    #[doc = include_str!("resource_bridge.doc/next_task_id.md")]
    fn next_task_id(&mut self) -> TaskId {
        let id = TaskId::from_raw(self.next_task);
        self.next_task += 1;
        id
    }

    #[doc = include_str!("resource_bridge.doc/release_evicted.md")]
    fn release_evicted(&mut self, evicted: Vec<Box<str>>) {
        for url in evicted {
            if let Some(handle) = self.image_handles.remove(&url) {
                self.hub.release(handle);
            }
        }
    }

    #[doc = include_str!("resource_bridge.doc/set_image_cache_budget.md")]
    pub(crate) fn set_image_cache_budget(&mut self, budget: CacheBudget) -> usize {
        let evicted = self.image_cache.set_budget(budget);
        let count = evicted.len();
        self.release_evicted(evicted);
        count
    }

    #[doc = include_str!("resource_bridge.doc/image_cache_budget.md")]
    pub(crate) fn image_cache_budget(&self) -> CacheBudget {
        self.image_cache.budget()
    }

    #[doc = include_str!("resource_bridge.doc/set_image_eviction_policy.md")]
    pub(crate) fn set_image_eviction_policy(&mut self, policy: EvictionPolicy) {
        self.image_cache.set_policy(policy);
    }

    #[doc = include_str!("resource_bridge.doc/image_cache_statistics.md")]
    pub(crate) fn image_cache_statistics(&self) -> CacheStatistics {
        self.image_cache.statistics()
    }

    #[doc = include_str!("resource_bridge.doc/resource_count.md")]
    pub(crate) fn resource_count(&self) -> usize {
        self.hub.len()
    }

    #[doc = include_str!("resource_bridge.doc/cached_counts.md")]
    pub(crate) fn cached_counts(&self) -> (usize, usize) {
        (self.image_cache.len(), self.vector_cache.len())
    }

    #[doc = include_str!("resource_bridge.doc/bookkeeping_counts.md")]
    #[cfg(test)]
    pub(crate) fn bookkeeping_counts(&self) -> (usize, usize) {
        (self.image_handles.len(), self.image_referrers.len())
    }

    #[doc = include_str!("resource_bridge.doc/invalidate_dpi_variants.md")]
    pub(crate) fn invalidate_dpi_variants(&mut self) {
        let resident: Vec<Box<str>> = self.image_cache.images().keys().cloned().collect();
        self.image_cache.clear();
        self.release_evicted(resident);
    }

    #[doc = include_str!("resource_bridge.doc/mark_image_visibility.md")]
    pub(crate) fn mark_image_visibility(
        &mut self,
        computed: &ComputedStyles,
        tree: &RetainedTree,
        boxes: &LayoutBoxes,
        viewport: Rect,
    ) {
        let mut referenced: HashMap<Box<str>, VisibilityPriority> = HashMap::new();
        for (node, style) in computed {
            if let Some(url) = style
                .get(PropertyId::BackgroundImage)
                .and_then(ComputedValue::as_background_image_url)
            {
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
    }

    #[doc = include_str!("resource_bridge.doc/retain_referenced.md")]
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
        self.image_referrers
            .retain(|url, _| live.contains(url.as_ref()));
    }

    #[doc = include_str!("resource_bridge.doc/resolve_background_images.md")]
    pub(crate) fn resolve_background_images(
        &mut self,
        computed: &ComputedStyles,
        device_scale: f32,
        report: &ResourceReporting<'_>,
    ) {
        // Both a loader and a decoder are required to actually fetch and decode.
        // Without either, a `url()` background cannot become `ready` — but the
        // reference is still a real resource request and must REACH the hub (spec
        // §5.15 "loading/ready/failed states;"): the kit does not fetch it by
        // itself, so with no provider the request lands in a terminal `failed`
        // state rather than never existing. A handle parked outside the hub is
        // indistinguishable from a URL that was never referenced. Each URL is
        // registered with the hub once and named once on the diagnostics channel.
        if self.loader.is_none() || self.decoder.is_none() {
            let urls: Vec<Box<str>> = computed
                .values()
                .filter_map(|style| {
                    style
                        .get(PropertyId::BackgroundImage)
                        .and_then(ComputedValue::as_background_image_url)
                        .map(Box::<str>::from)
                })
                .collect();
            for url in urls {
                // `image_handles` is the persistent "already requested" set, so a
                // URL is registered and reported exactly once however many frames
                // reference it (the diagnostics dedup on their own channel too).
                if self.image_handles.contains_key(&url) {
                    continue;
                }
                let source = ResourceSource::file(url.clone());
                let handle = self
                    .hub
                    .request(ResourceRequest::new(source, ResourceKind::Image));
                self.image_handles.insert(url.clone(), handle);
                // No provider can ever service this request: drive it straight to a
                // terminal `failed` state, so a caller of `state()` can tell it is
                // dead rather than parked in `loading` behind a slow network.
                if let Some(version) = self.hub.begin_load(handle) {
                    let _ = self.hub.fail(
                        handle,
                        version,
                        ResourceError::SourceUnavailable(
                            "no host loader is installed to service this source".into(),
                        ),
                    );
                }
                self.report_unresolvable_source(&url, report.sink);
            }
            return;
        }
        for (node, style) in computed {
            let Some(url) = style
                .get(PropertyId::BackgroundImage)
                .and_then(ComputedValue::as_background_image_url)
            else {
                continue;
            };
            // Attribute the resource to the element whose `background-image`
            // referenced it, so a later load failure names the node (spec §1.6).
            self.image_referrers.entry(url.into()).or_insert(*node);
            // Asking through `record_lookup` is what makes this frame's reference
            // count as a hit and stamp the entry's recency, so eviction can tell a
            // live image from a stale one (spec §5.19).
            if self.image_cache.record_lookup(url) || self.image_handles.contains_key(url) {
                // Already resolved, or already attempted (and failed): do not retry.
                continue;
            }
            let key: Box<str> = url.into();
            // A `url()` is resolved by the loader port from a plain path/source.
            let source = ResourceSource::file(key.clone());
            let handle = self
                .hub
                .request(ResourceRequest::new(source.clone(), ResourceKind::Image));
            self.image_handles.insert(key.clone(), handle);
            let Some(version) = self.hub.begin_load(handle) else {
                continue;
            };
            // Load → decode → build the SceneImage; any failure fails the handle
            // and caches nothing (the image is not painted).
            self.load_image(key, handle, version, device_scale);
        }
    }

    #[doc = include_str!("resource_bridge.doc/resolve_element_images.md")]
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
            if self.image_cache.record_lookup(&key) || self.image_handles.contains_key(&key) {
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

    #[doc = include_str!("resource_bridge.doc/resolve_element_vectors.md")]
    pub(crate) fn resolve_element_vectors(&mut self, tree: &RetainedTree) {
        // A loader and a VECTOR decoder are both required; without either, an SVG
        // source honestly resolves to nothing.
        if self.loader.is_none() || self.vector_decoder.is_none() {
            return;
        }
        let sources: Vec<Box<str>> = tree
            .iter()
            .filter_map(|(_, node)| node.attributes().image_source().map(Box::<str>::from))
            .filter(|src| is_vector_source(src))
            .collect();
        for key in sources {
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

    #[doc = include_str!("resource_bridge.doc/load_image.md")]
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
        let outcome = move || {
            ResourceOutcome::load_image(&*loader, &*decoder, &candidates, key, handle, version)
        };
        if let Some(executor) = self.blocking_executor.clone() {
            let tx = self.tx.clone();
            let id = self.next_task_id();
            executor.run_blocking(Box::new(move || {
                // The receiver is gone only if the runtime was dropped mid-load;
                // there is nothing left to tell, so the send is allowed to fail.
                let _ = tx.send(TaskResult::new(id, TaskScope::Runtime, outcome()));
            }));
        } else {
            // No executor: QUEUE the load rather than running it here. It runs on
            // the frame thread — but after the frame phase — so the loader is never
            // invoked inside `render_now` (§3.4, §5.15). `image_handles` already
            // marked this URL pending, so a later frame will not re-queue it.
            self.deferred_loads.push(Box::new(outcome));
        }
    }

    #[doc = include_str!("resource_bridge.doc/load_vector.md")]
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
        let outcome = move || {
            ResourceOutcome::load_vector(&*loader, &*decoder, &source, key, handle, version)
        };
        if let Some(executor) = self.blocking_executor.clone() {
            let tx = self.tx.clone();
            let id = self.next_task_id();
            executor.run_blocking(Box::new(move || {
                let _ = tx.send(TaskResult::new(id, TaskScope::Runtime, outcome()));
            }));
        } else {
            // No executor: defer the run out of the frame phase, exactly as
            // `load_image` does — the loader must not be reached inside
            // `render_now` (§3.4, §5.15).
            self.deferred_loads.push(Box::new(outcome));
        }
    }

    #[doc = include_str!("resource_bridge.doc/size_replaced_elements.md")]
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
                    .image_cache
                    .images()
                    .get(src)
                    .map(SceneImage::logical_size)
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
            if let Some(style) = layout_styles.get_mut(&node)
                && style.width.is_auto()
                && style.height.is_auto()
            {
                style.width = Dimension::Px(iw);
                style.height = Dimension::Px(ih);
            }
        }
    }

    #[doc = include_str!("resource_bridge.doc/report_unresolvable_source.md")]
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

    #[doc = include_str!("resource_bridge.doc/report_failure.md")]
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
    #[doc = include_str!("resource_bridge.doc/fmt.md")]
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

#[doc = include_str!("resource_bridge.doc/image_priority.md")]
fn image_priority(boxes: &LayoutBoxes, node: NodeId, viewport: Rect) -> VisibilityPriority {
    match boxes.get(&node) {
        Some(layout_box) if rects_intersect(layout_box.bounds, viewport) => {
            VisibilityPriority::Visible
        }
        _ => VisibilityPriority::Offscreen,
    }
}

#[doc = include_str!("resource_bridge.doc/rects_intersect.md")]
fn rects_intersect(a: Rect, b: Rect) -> bool {
    a.origin.x < b.right()
        && b.origin.x < a.right()
        && a.origin.y < b.bottom()
        && b.origin.y < a.bottom()
}

#[doc = include_str!("resource_bridge.doc/intrinsic_of.md")]
#[allow(clippy::cast_precision_loss)]
fn intrinsic_of(image: &SceneImage) -> Size {
    Size::new(image.width() as f32, image.height() as f32)
}

#[doc = include_str!("resource_bridge.doc/localized_diagnostic_message.md")]
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

#[doc = include_str!("resource_bridge.doc/is_vector_source.md")]
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
        bridge.resolve_element_vectors(&tree);
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
