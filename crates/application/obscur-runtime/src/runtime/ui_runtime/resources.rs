#![doc = include_str!("resources.doc/_module.md")]

use std::sync::Arc;

use obscur_resources::cache::{CacheBudget, CacheStatistics, EvictionPolicy};
use obscur_resources::port::{ResourceDecoder, ResourceLoader, VectorDecoder};
use obscur_task::BlockingExecutor;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("resources.doc/set_resource_providers.md")]
    pub fn set_resource_providers(
        &mut self,
        loader: Arc<dyn ResourceLoader + Send + Sync>,
        decoder: Arc<dyn ResourceDecoder + Send + Sync>,
    ) {
        self.resources.set_providers(loader, decoder);
    }

    #[doc = include_str!("resources.doc/set_blocking_executor.md")]
    pub fn set_blocking_executor(&mut self, executor: Arc<dyn BlockingExecutor + Send + Sync>) {
        self.resources.set_blocking_executor(executor);
    }

    #[doc = include_str!("resources.doc/set_vector_decoder.md")]
    pub fn set_vector_decoder(&mut self, decoder: Arc<dyn VectorDecoder + Send + Sync>) {
        self.resources.set_vector_decoder(decoder);
    }

    #[doc = include_str!("resources.doc/set_image_cache_budget.md")]
    pub fn set_image_cache_budget(&mut self, budget: CacheBudget) -> usize {
        self.resources.set_image_cache_budget(budget)
    }

    #[doc = include_str!("resources.doc/image_cache_budget.md")]
    #[must_use]
    pub fn image_cache_budget(&self) -> CacheBudget {
        self.resources.image_cache_budget()
    }

    #[doc = include_str!("resources.doc/set_image_eviction_policy.md")]
    pub fn set_image_eviction_policy(&mut self, policy: EvictionPolicy) {
        self.resources.set_image_eviction_policy(policy);
    }

    #[doc = include_str!("resources.doc/image_cache_statistics.md")]
    #[must_use]
    pub fn image_cache_statistics(&self) -> CacheStatistics {
        self.resources.image_cache_statistics()
    }

    #[doc = include_str!("resources.doc/resource_count.md")]
    #[must_use]
    pub fn resource_count(&self) -> usize {
        self.resources.resource_count()
    }

    #[doc = include_str!("resources.doc/cached_resource_counts.md")]
    #[must_use]
    pub fn cached_resource_counts(&self) -> (usize, usize) {
        self.resources.cached_counts()
    }
}
