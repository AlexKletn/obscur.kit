Expands the `flex` shorthand into `flex-grow` / `flex-shrink` / `flex-basis`
(CSS Flexible Box Layout Level 1 §7.1).

The grammar is `none | [ <'flex-grow'> <'flex-shrink'>? || <'flex-basis'> ]`.
The defaults for an OMITTED component are deliberately not the longhands'
initial values — the spec's own note reads "The initial values of the flex
longhands are equivalent to `flex: 0 1 auto`. This differs from their defaults
when omitted in the flex shorthand (effectively `1 1 0px`)". So:

* an omitted `<'flex-grow'>` is `1`, and an omitted `<'flex-shrink'>` is `1`;
* an omitted `<'flex-basis'>` has "its specified value is 0" — `0px`, NOT
  `auto`;
* `flex: none` "expands to `0 0 auto`";
* `flex: <number>` is therefore `<number> 1 0px` — `flex: 1` is NOT
  `flex: 1 1 auto`;
* `flex: auto` is `1 1 auto` (a lone `<'flex-basis'>` component).

`initial` / `inherit` / `unset` are CSS-wide keywords rather than shorthand
components, so they are handed to each longhand unchanged (`flex: initial`
thereby lands on the longhand initial values, `0 1 auto`).

Because "a unitless zero that is not already preceded by two flex factors must
be interpreted as a flex factor", a bare `<number>` is read as a flex factor in
every position except the third component of a three-value form, where the only
number a `<'flex-basis'>` accepts is `0`. A negative factor is outside the
`<number [0,∞]>` range. Any value this grammar rejects degrades the whole
shorthand to a single [`CompiledDeclaration::Unsupported`] rather than being
dropped silently (§4.13).
