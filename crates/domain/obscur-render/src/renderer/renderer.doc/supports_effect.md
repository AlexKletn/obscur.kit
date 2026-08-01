The honest [`SupportLevel`] at which this backend reproduces `effect`, so a
host can find out what it will actually get before it asks for it
(spec §5.2).

The vocabulary is closed ([`SceneEffect`]) precisely so this table can be
audited: every arm is a claim about what the backend's own paint path does
with that effect's parameters, and the compiler refuses a table with a gap.
The levels mean:

- [`Native`](SupportLevel::Native): reproduced by a first-class primitive of
  the backend.
- [`Emulated`](SupportLevel::Emulated): reproduced by other means; the
  effect's parameters are all honored and only rasterization differs
  (the §5.2 determinism boundary).
- [`Limited`](SupportLevel::Limited): reproduced under a restriction the
  backend documents on this method — some input of the effect is not
  modeled.
- [`Unavailable`](SupportLevel::Unavailable): **not** reproduced. Whatever
  the backend draws instead is a fallback, not the effect. Ignoring a
  parameter entirely is this level, not `Emulated`.
