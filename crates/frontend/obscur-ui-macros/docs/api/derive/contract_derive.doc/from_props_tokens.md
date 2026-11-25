Lowers the field into its `<ident>: <expr>` initialiser for the generated
[`FromProps`] impl.

The three shapes mirror [`contract_tokens`](Self::contract_tokens) exactly,
so what the contract *declares* and what the decoder *reads* cannot drift:

* a skipped or event field is not a prop at all, so it is not decoded — it
  takes its [`Default`]. (This is why deriving `Contract` on a struct with
  a non-`Default` skipped field is a compile error, which is the honest
  outcome: there is nothing to decode it from.)
* an `Option<T>` decodes optionally — absent or `Null` yields `None`, but a
  present-yet-unreadable value is still an error;
* a `#[contract(default)]` field falls back to its [`Default`] when absent;
* every other field is required.

The target type is never named: it is inferred from the field being
initialised, so no `Option<T>` inner-type extraction is needed.

[`FromProps`]: obscur_view::component::FromProps
[`Default`]: ::core::default::Default
