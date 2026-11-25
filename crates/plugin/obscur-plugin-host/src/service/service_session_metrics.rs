//! Call quota diagnostics for an authenticated service session.

/// Current in-flight calls and cumulative quota rejection counts for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceSessionMetrics {
    /// Number of calls currently holding a concurrency slot.
    pub in_flight: usize,
    /// Number of calls rejected by the rolling rate limiter.
    pub rate_limit_rejections: u64,
    /// Number of calls rejected because every concurrency slot was in use.
    pub concurrency_limit_rejections: u64,
    /// Number of admitted calls that took at least the configured threshold before completion or cancellation.
    pub slow_calls: u64,
}
