Open a **desktop surface** — a panel, dock, bar, wallpaper or overlay — instead
of an ordinary window.

This is the whole of the difference. Everything else about the `App` is
unchanged: the same [`view`](Self::view), the same CSS, the same commands, the
same virtual lists and row clicks, the same runtime underneath. What changes is
where the surface ends up — anchored to a screen edge, in a depth band, outside
the window stack — and that is described by
[`LayerSettings`](crate::windowing::layer::LayerSettings), which names no
platform.

```ignore
use obscur_kit::App;
use obscur_kit::windowing::layer::LayerSettings;

App::new("status bar")
    .app_id("com.example.bar")
    .layer(LayerSettings::top_bar(32))
    .stylesheet(".bar { background-color: #1c2233; color: #eef }")
    .view(build_bar)
    .run()?;
```

The application's [`app_id`](Self::app_id) becomes the surface's identity to the
desktop — the string a compositor rule matches on and its tooling lists. There
is no second name to set.

Where the platform has no desktop surfaces at all,
[`run`](Self::run) returns
[`AppError::LayerUnsupported`](crate::AppError::LayerUnsupported) — a capability
answer to branch on, not a crash, and the cue to fall back to a window by simply
not calling this.
