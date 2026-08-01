#![doc = include_str!("mod.doc/_module.md")]
mod navigation_result;
pub use navigation_result::NavigationResult;
mod route_registry;
pub use route_registry::RouteRegistry;
mod router;
pub use router::Router;
