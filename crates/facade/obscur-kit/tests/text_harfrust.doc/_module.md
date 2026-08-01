Proves the real HarfRust text shaper is wired through the facade: the
`text-harfrust` feature re-exports the adapter, `KitBootstrap` builds one from
font bytes, and installing it on a [`UiRuntime`] via the public
`set_text_shaper` seam changes the composed `DrawText` run.

This whole file only compiles under the `text-harfrust` feature (implied by
`full`); it is skipped otherwise.
