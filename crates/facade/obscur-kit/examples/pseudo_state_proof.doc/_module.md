**Proof: the `:disabled` and `:checked` pseudo-classes bite from widget state.**

Renders four BARE built-in controls through the software [`CpuRenderer`] in one
`target/pseudo_state_proof.png`:

- an **enabled** [`Button`] beside a **disabled** one, and
- an **unchecked** [`Checkbox`] beside a **checked** one.

The runtime has the toolkit's UA default stylesheet installed at the lowest
cascade origin plus a small author sheet whose `button:disabled` and
`checkbox:checked` rules restyle the state. Because each widget now emits the
reserved `DISABLED`/`CHECKED` structural-state attribute (which survives
reconcile and feeds the selector matcher's `PseudoState`), the disabled button
and the checked checkbox visibly differ from their enabled/unchecked peers —
proving the state pseudo-classes are wired end-to-end.

Run it:
`cargo run -p obscur-kit --example pseudo_state_proof \
  --features "render-cpu text-harfrust text-unicode"`
