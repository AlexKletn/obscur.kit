The authored stylesheet: a clean dark theme keyed to the widgets' stable CSS
types (`button` / `checkbox` / `input`) and the composed elements' classes,
with live `:hover` / `:focus` / `:active` / `:checked` / `:disabled` rules.
Returned as a `&'static str` so both drivers compile it through the real
`obscur-css` frontend.
