Binds a loopback DevTools endpoint and serves it on a background thread,
so an external `obscur-devtools` client can inspect this live window — its
tree/layout/styles and the `Rendering` frame-perf snapshot (spec §7.6/§7.9)
— and *subscribe* to the frame/focus/diagnostic events it publishes.
Local-only and token-authenticated; a bind failure is non-fatal.
