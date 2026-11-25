Compiles `sources` in order and installs the result on `runtime`: the
matchable rules via
[`set_stylesheets`](crate::runtime::UiRuntime::set_stylesheets), and the
sheet's `transition`/`@keyframes` via
[`set_css_motion`](crate::runtime::UiRuntime::set_css_motion) so the
runtime actually drives them (§4.17).

Each source is parsed only if the cache has no entry at its
[`version`](CssSource::version); an
[`ANONYMOUS`](obscur_css::source::SourceId::ANONYMOUS) source has no stable
identity and is therefore always re-parsed. Everything downstream of the
parse (compile, lower, install) runs every call — this caches parsing, which
is what §4.12 specifies.

A malformed sheet compiles to whatever it recovered — the rejected parts
never reach the runtime, and the surviving rules do (spec §1.6 "failures
should be localized to the smallest safe scope"). Each rejected part is
**also** raised as a `Style`-subsystem [`Diagnostic`] on the runtime's
§1.6 channel via
[`report_diagnostic`](crate::runtime::UiRuntime::report_diagnostic): a
syntactic error (`color: ;`) and a recognised-but-unsupported construct
(`width: 10qq`, `float: left`) each report one diagnostic. The runtime
receives only compiled rules and never a sheet's source, so this — the one
place holding both the errors and the runtime — is where they reach the
channel. The same lists remain readable on [`errors`](Self::errors) and
[`unsupported`](Self::unsupported).
