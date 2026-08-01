Umbrella facade for Obscur Kit: a stable prelude, a high-level bootstrap, and
curated re-exports of the toolkit's crates (spec §2.9).

Applications depend on this one crate instead of the individual foundation,
domain, application, and adapter crates. It contains no domain implementation
and is never used as a dependency of internal crates.

# Re-exports

Every toolkit crate is re-exported under a short module alias, so
`obscur_kit::scene::Color` reaches the same type as `obscur_scene::color::Color`.
Optional adapters are re-exported only when their Cargo feature is enabled,
and the platform system-service provider is selected automatically by target.

# Features

The `default` set is portable (`render-cpu`, `task-smol`, `resource-fs`,
`cache-fs`) and needs no display or GPU. The `full` set additionally enables
`render-vello`, `window-winit`, `task-tokio`, `accesskit`, and `text-harfrust`
(the real HarfRust text shaper).
