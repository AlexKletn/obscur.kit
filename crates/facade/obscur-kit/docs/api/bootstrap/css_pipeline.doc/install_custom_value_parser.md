Arms the runtime's reading of a custom property's declared value, once.

A custom property keeps the text it was written with, because CSS gives it no
grammar until a `var()` puts it on a property — and the property is only known
at computed-value time, inside the runtime. So the grammar goes in the same
way the `style` attribute's compiler does, and for the same reason (§1.11).
The consequence worth knowing: `font-family: var(--font-body)` resolves to a
family list exactly where this ran, and computes to `unset` where it did not.

What is installed is a
[`CustomValueCache`](obscur_css::compiler::CustomValueCache) rather than the
bare lowering function. Reading a declared value is a pure function of the text
and the property it landed on, and the resolver asks the same question once per
`var()` per node per restyle — so the grammar was being run over the same few
bytes hundreds of times a frame, which is what made a themed sheet cost double.
The cache is arrived at once, here, and lives as long as the runtime does.

Guarded on [`UiRuntime::has_custom_value_parser`], because
[`CssPipeline::install`](super::CssPipeline::install) runs again on every live
stylesheet replacement and the grammar does not change between sheets. The same
guard is what lets a second caller arm the slot without ever taking a host's own
lowering away from it.

## Why this is `pub(super)` — the funnel, and where it is called

This was private, and the CSS pipeline was its only caller, for as long as a
single-component custom property survived compilation as a typed value:
`--obscur-sys-window-bg: #202020` came out a `PropertyValue::Color`, and reading
it back through `var()` needed no grammar at all. Once a custom property began
keeping its raw tokens — and once a substituted value that does not fit its
property began computing to `unset` rather than to something approximate — this
call became load-bearing for *every* `var()`, including the ones the kit writes
itself. [`KitBootstrap::install_system_theme`](super::KitBootstrap::install_system_theme)
compiles a whole `:root { --obscur-sys-…: … }` block into the runtime, so a host
that installed the system theme and never named [`CssPipeline`](super::CssPipeline)
got `unset` for every desktop colour it read.

The funnel chosen is **each `KitBootstrap` entry point that installs CSS into a
runtime** — [`install_default_stylesheet`](super::KitBootstrap::install_default_stylesheet)
and [`install_system_theme`](super::KitBootstrap::install_system_theme), and so
also `install_system_theme_from`, which goes through the latter.

Not `install_system_theme` alone, because the theme is not the only way a host
reaches this wall: a host that installs the UA sheet and then hands
`set_stylesheets` its own hand-compiled `:root { --brand: … }` hits exactly the
same one, and that shape is the commoner of the two. The defect is not "the
system theme has variables in it" — it is "a runtime the kit set up cannot read
a custom property back", and the UA sheet is the first moment the kit is
demonstrably setting up CSS on that runtime.

Not every `KitBootstrap` path that touches a runtime either. Arming a CSS
grammar as a side effect of `install_accessibility` or
`install_blocking_executor` would couple two things that have nothing to say to
each other, and a reader of either call would have no way to expect it. The line
is CSS: the bootstrap installs the grammar for the CSS it installs, and nothing
else.

Layering is why the funnel has to be here at all rather than in the runtime:
`obscur-runtime` may not depend on `obscur-css` (§1.11), so the facade is the
only tier that can hand one to the other.
