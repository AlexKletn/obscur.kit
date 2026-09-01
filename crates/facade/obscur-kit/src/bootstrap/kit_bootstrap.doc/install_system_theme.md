Install the desktop's appearance as CSS custom properties, and hand back what
was installed.

The UA sheet and a `:root { --obscur-sys-…: … }` block, compiled together and
set as the runtime's default stylesheet — so the variables sit at the
**system-default origin**, which is the right relationship: the desktop is
making a suggestion, and an author who writes anything at all outranks it.

Returns the [`ThemeVars`](crate::ThemeVars) so a host can see what the desktop
actually answered. An empty set is a real answer — see
[`ThemeVars`](crate::ThemeVars) for what "the desktop did not say" means and for
the `var()` limits that decide how a sheet must be written to read these at all.

Reading the appearance is not this call's business: an
[`AppearanceProvider`](obscur_system::appearance::AppearanceProvider) does that,
and calling this again with a fresh reading is how a host follows a theme change.

An empty appearance is installed without complaint about the *desktop* — there
may not have been one to ask — but it does warn `platform.appearance.empty`,
since a stylesheet reading `var(--obscur-sys-accent)` after this call gets
nothing and the reason is worth one line. Use
[`install_system_theme_from`](Self::install_system_theme_from) to have the
distinction between "no source" and "the source said nothing" drawn for you.

## The variables are readable without a CSS pipeline

This call writes `var()`-bearing CSS into the runtime, so it also arms the
runtime's custom-property grammar when nothing has yet — the same guarded
install [`CssPipeline`](super::CssPipeline) performs, and it never displaces a
parser a host chose first. Without it, `var(--obscur-sys-window-bg)` computes to
`unset` for every host that installed the system theme and never named the CSS
pipeline: a custom property keeps the text it was declared with, and reading one
back is what needs the grammar. The funnel and why it is drawn where it is are
written down beside that installer.
