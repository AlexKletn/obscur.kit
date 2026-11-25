Send the toolkit's diagnostics somewhere of your choosing.

Replaces the default, which is
[`StderrDiagnostics`](crate::bootstrap::StderrDiagnostics) in a debug build and
silence in a release one.

Worth doing even when the default would suit: the toolkit reports a great deal
that is only visible this way — a CSS declaration it could not compile, a
component that refused its props, a duplicate key rebuilding a subtree every
frame — and none of it is an error, so none of it stops anything.

```ignore
use obscur_kit::bootstrap::StderrDiagnostics;
use obscur_kit::diagnostics::DiagnosticSeverity;

App::new("app")
    .diagnostics(StderrDiagnostics::from_severity(DiagnosticSeverity::Info))
    .view(build)
    .run()?;
```
