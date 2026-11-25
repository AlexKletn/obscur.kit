This element's computed `writing-mode`.

Cascaded value first, then the parent's computed one (`writing-mode`
inherits — css-writing-modes-4 §3), then the CSS initial `horizontal-tb`.
Resolved early, next to [`resolves_rtl`], because the
logical→physical box-model mapping ([`PropertyId::physical_under`]) depends
on it. An absent value is `horizontal-tb`, under which every mapping is
byte-for-byte the pre-`writing-mode` behaviour.
