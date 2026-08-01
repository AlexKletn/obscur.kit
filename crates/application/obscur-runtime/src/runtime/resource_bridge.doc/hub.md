The pure lifecycle/versioning registry for requested image resources (spec
§5.15). It stores only lifecycle + metadata, never the decoded pixels — the
pixels live in [`image_cache`](Self::image_cache) — so the two are kept in
step by URL.
