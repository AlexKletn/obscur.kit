Every display a desktop surface could be opened on, asked before opening one.

Closes the ordering problem in
[`LayerSettings::with_display`](crate::windowing::layer::LayerSettings::with_display):
that method takes a `DisplayId`, an id only means something once a backend has
enumerated its outputs, and the settings must be complete before
[`run`](Self::run) opens anything. This looks first.

```ignore
use obscur_kit::App;
use obscur_kit::windowing::layer::LayerSettings;

let mut settings = LayerSettings::top_bar(32);
if let Some(display) = App::displays()?.iter().find(|d| d.name() == "eDP-1") {
    settings = settings.with_display(display.id());
}
```

Left unset, the platform chooses, which on a multi-head desktop means whichever
output had focus when the surface was created — rarely what a panel wants, and
actively wrong for an on-screen keyboard, whose keys would land on a different
screen from the field being typed into.

Opens and drops its own short-lived connection, so it may be called before
anything else and costs nothing afterwards.

# Errors

[`LayerUnsupported`](crate::AppError::LayerUnsupported) where the desktop has no
layer surfaces at all — the same answer [`run`](Self::run) would give, so a host
can branch once — and [`Layer`](crate::AppError::Layer) where the platform has
them and the query itself failed.
