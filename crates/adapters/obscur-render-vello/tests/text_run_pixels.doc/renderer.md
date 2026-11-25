A renderer on this machine's adapter, with the face registered — or a panic.

Every claim in this file is a claim about pixels a GPU produced, so absence of an
adapter is not a state in which any of it can be answered. It used to be handed
back as `None`, and each test then printed a line and returned — that is, passed.
"No GPU here" and "checked, and correct" reported identically, which is the
defect this file's own module note warns about in the code it tests.

Absence is therefore an error, and the tests carry `#[ignore]` so that error only
fires where it should:

* `#[ignore]` is the default. On a GPU-less runner `cargo test` lists these in
  the **ignored** column — visibly not "passed".
* `-- --ignored` is a statement that this machine has an adapter. If it has
  none, the run is broken and says so.

The named command is written on each `#[ignore]` reason.
