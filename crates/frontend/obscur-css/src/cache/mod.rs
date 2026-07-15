//! Cache module: parse each CSS source once per revision (spec §4.12).
mod stylesheet_cache;
pub use stylesheet_cache::StylesheetCache;
