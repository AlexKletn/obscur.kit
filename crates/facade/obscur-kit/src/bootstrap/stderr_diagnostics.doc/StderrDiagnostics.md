Writes warnings and worse to stderr, one line each.

Exists because of the failure it prevents. The runtime's own default sink
discards everything, which is correct for a library — nothing embedded should
print — and wrong for a standalone process: an application that never went
looking for a sink saw none of the toolkit's reporting, and a toolkit that
reports carefully into a sink nobody installed is indistinguishable from one
that does not report at all.

[`App`](crate::App) installs this by default **in a debug build**, so the first
run of an application shows what the boot found. A release build keeps the
silent default, and either can be replaced with
[`App::diagnostics`](crate::App::diagnostics).
