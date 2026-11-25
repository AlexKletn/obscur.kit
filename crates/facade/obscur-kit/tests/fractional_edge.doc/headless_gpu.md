An off-screen wgpu device, or a panic naming the adapter that is missing.

The caller is `#[ignore]`d, so the only way to reach this function is to ask for
the test by name — which is a statement that this machine has an adapter. If it
has none, the run is broken and says so, rather than printing a line and
returning, which is a pass wearing a skip's clothes.

The two states are kept distinct on purpose. `#[ignore]` is what a GPU-less
runner sees: the test lands in the **ignored** column of `cargo test`, a number
that is visibly not "passed". `-- --ignored` is what a machine with a GPU runs,
and there absence is an error.
