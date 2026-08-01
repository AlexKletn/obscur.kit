Expands the `background` shorthand — the minimal image / color subset the
domain models today (spec CSS Backgrounds and Borders §3).

Only two authored forms are recognised: a single `linear-gradient(...)` maps
to `background-image`, and a single `<color>` maps to `background-color`.
Every other `background` value — a `url(...)` image, position / size / repeat /
attachment components, or multiple comma-separated layers — is out of scope and
degrades the whole declaration to a single [`CompiledDeclaration::Unsupported`]
rather than being dropped silently.
