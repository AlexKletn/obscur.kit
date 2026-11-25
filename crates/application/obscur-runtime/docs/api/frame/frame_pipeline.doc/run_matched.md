Runs one synchronous frame that **matches authored stylesheets** against
the reconciled tree instead of taking pre-cascaded [`StyleInputs`].

This is the integrating path that lets authored CSS reach paint. It runs
the same phases as [`run`](Self::run) with one change to the style stage:

1. **Reconcile** — identical to [`run`](Self::run).
2. **Match** — the [`StyleMatchingCoordinator`] matches `sheets` against
   the just-reconciled tree, cascading each element's winning per-property
   values into the [`StyleInputs`] that [`run`](Self::run) would otherwise
   receive from the caller.
3. **Style / Layout / Compose** — identical to [`run`](Self::run).

`sheets` must be ordered by ascending cascade origin (system-default
first, inline last). With an empty `sheets` slice no rule matches, the
inputs are empty, and the result matches `run` with empty inputs.
