#![doc = include_str!("../../../docs/api/runtime/ui_runtime/resources.doc/_module.md")]

use std::sync::Arc;

use obscur_resources::cache::{CacheBudget, CacheStatistics, EvictionPolicy};
use obscur_resources::port::{ResourceDecoder, ResourceLoader, VectorDecoder};
use obscur_task::BlockingExecutor;

use crate::frame::FrameReason;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/resources.doc/set_resource_providers.md")]
    pub fn set_resource_providers(
        &mut self,
        loader: Arc<dyn ResourceLoader + Send + Sync>,
        decoder: Arc<dyn ResourceDecoder + Send + Sync>,
    ) {
        self.resources.set_providers(loader, decoder);
        if self.last_layout_inputs.take().is_some() {
            self.scheduler
                .request_reason(FrameReason::ResourceCompletion);
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/resources.doc/set_blocking_executor.md")]
    pub fn set_blocking_executor(&mut self, executor: Arc<dyn BlockingExecutor + Send + Sync>) {
        self.resources.set_blocking_executor(executor);
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/resources.doc/set_vector_decoder.md")]
    pub fn set_vector_decoder(&mut self, decoder: Arc<dyn VectorDecoder + Send + Sync>) {
        self.resources.set_vector_decoder(decoder);
        if self.last_layout_inputs.take().is_some() {
            self.scheduler
                .request_reason(FrameReason::ResourceCompletion);
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/resources.doc/set_image_cache_budget.md")]
    pub fn set_image_cache_budget(&mut self, budget: CacheBudget) -> usize {
        if budget == self.resources.image_cache_budget() {
            return 0;
        }
        let evicted = self.resources.set_image_cache_budget(budget);
        if self.last_layout_inputs.is_some() {
            self.scheduler
                .request_reason(FrameReason::ResourceCompletion);
        }
        evicted
    }

    /// The memory budget the decoded-image cache is held under (spec §5.19).
    #[must_use]
    pub fn image_cache_budget(&self) -> CacheBudget {
        self.resources.image_cache_budget()
    }

    /// Installs the strategy the decoded-image cache evicts by (spec §5.19
    /// *eviction*), replacing [`EvictionPolicy::VisibilityAware`] (the default).
    pub fn set_image_eviction_policy(&mut self, policy: EvictionPolicy) {
        self.resources.set_image_eviction_policy(policy);
    }

    /// The decoded-image cache's live metrics: hits, misses, evictions,
    /// insertions, resident entries and resident bytes (spec §5.19 *`DevTools`
    /// metrics*).
    #[must_use]
    pub fn image_cache_statistics(&self) -> CacheStatistics {
        self.resources.image_cache_statistics()
    }

    /// How many resources have been requested this session — loaded, in flight or
    /// failed (spec §7.6 `Resources`).
    #[must_use]
    pub fn resource_count(&self) -> usize {
        self.resources.resource_count()
    }

    /// Returns the current version of a resource registered by this runtime.
    #[must_use]
    pub fn resource_version(
        &self,
        id: obscur_identity::ResourceId,
    ) -> Option<obscur_resources::state::ResourceVersion> {
        self.resources.resource_version(id)
    }

    /// How many decoded images and parsed vector documents are cached
    /// (spec §7.6 `Caches`).
    #[must_use]
    pub fn cached_resource_counts(&self) -> (usize, usize) {
        self.resources.cached_counts()
    }
}
