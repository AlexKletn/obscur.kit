The typed value a property can hold.

[`PropertyValue`] is the typed result the cascade works with — never CSS
source text. Keyword-valued properties (for example `display: block`) store
their keyword as an [`InternedSymbol`] handle, not a string: matching is done
on the interned identity, not by comparing characters.

Two kinds of value are resolved *before* computed values are produced:

* the CSS-wide keywords [`PropertyValue::Inherit`], [`PropertyValue::Initial`]
  and [`PropertyValue::Unset`], and
* an unresolved custom-property reference [`PropertyValue::CustomRef`]
  (`var(--name)`), which is substituted during custom-property resolution
  (§4.14) before computed-value resolution runs.

`calc()` / `min()` / `max()` / `clamp()` and relative/viewport units are
modelled: a plain relative unit is a [`PropertyValue::Length`] carrying the
relevant [`Length`] variant, and a math function is a
[`PropertyValue::Calc`] holding a symbolic [`LengthExpr`]. Both resolve to
concrete pixels at the layout/compute step against a
[`LengthContext`](crate::property::LengthContext) (spec §4.5, §4.13).
