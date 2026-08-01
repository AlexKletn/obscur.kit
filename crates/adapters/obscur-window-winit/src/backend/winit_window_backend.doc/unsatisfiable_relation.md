Returns why `relation` cannot be materialized by winit, or `None` if it can
(spec §6.1).

winit 0.30 has no owner relation and no modality: `with_parent_window` builds
a clipped *child* window rather than an owned floating one, and nothing at all
blocks input to sibling windows. Since `create_window` reports through a
`Result` and has no capability channel to degrade through, an unsatisfiable
relation is rejected with the reason rather than granted an id whose window
would silently be top-level and modeless.
