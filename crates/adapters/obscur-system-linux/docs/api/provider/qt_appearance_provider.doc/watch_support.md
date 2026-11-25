Always `Unavailable` — and this is the asymmetry worth knowing.

The source that states the most is the one that cannot say when it changes.
`kdeglobals` is a file, and a file emits nothing. Watching it would take
`inotify` or KDE's own `KConfigWatcher` bus traffic; both are dependencies this
adapter does not carry, and neither is worth hiding behind a `Native` that would
make a host stop polling.
