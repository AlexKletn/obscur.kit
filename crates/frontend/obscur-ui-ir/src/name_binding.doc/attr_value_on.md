Resolves the *value* an attribute written **on `tag`** contributes, the way
[`attr_symbol_on`] resolves the symbol it binds to.

It is the identity on every attribute but the numeric ones a control's scale is
made of — `min`/`max`/`step`/`value` on a slider, `value` on a progress —
where it normalises a literal that *names* a number into [`UiIrValue::Float`].

# Why a quoted number has to be a number

Markup spells its values with quotes: `<slider min="0" max="10" value="4"/>` is
the ordinary authored form, and the `.okui` grammar has no other one — only an
`{…}` expression escapes to a typed literal. But the tiers that read a slider's
scale accept `Int` and `Float` and nothing else (`obscur_view::view::slider`'s
`number`), so a `Text("0")` reaches them as *no value at all* and the control
falls back to the default `0..1`. The attribute would compile, travel the whole
tree, and mean nothing — the same silent nothing that made `<slider/>` worth
fixing in the first place.

So the two spellings converge here: `min="0"` and `min={0}` both lower to
`Float(0.0)`, which is also what `Slider::over(0.0, …)` writes. One authored
node, one built node, one shape.

A literal that does not name a number (`min="wide"`) is left exactly as written
rather than replaced with a guess: the reader below falls back to its own
default, and the value stays visible in the tree for anyone debugging why.
