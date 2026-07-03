//! Performance module.
mod performance_budget;
mod performance_span;

pub use performance_budget::PerformanceBudget;
pub use performance_span::PerformanceSpan;

mod budget_violation;
mod performance_counter;
