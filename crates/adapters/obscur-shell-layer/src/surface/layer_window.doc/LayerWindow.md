A live layer surface: the Wayland connection, the surface, and the queue that
drives them.

# Lifecycle

Opening does **not** give you a drawable surface. The protocol requires an
initial commit with no buffer, after which the compositor answers with a
configure carrying the real size — which is why
[`LayerEvent::Configured`](crate::event::LayerEvent::Configured) exists and
why [`is_configured`](Self::is_configured) gates the first frame. Drawing
before it is a protocol error, not a race.

# Driving it

```ignore
let mut window = LayerWindow::open(&LayerSurfaceConfig::top_bar("panel", 32))?;
while !window.is_closed() {
    for event in window.dispatch()? {
        // feed pointer events to the runtime, resize on Configured, …
    }
}
```

[`dispatch`](Self::dispatch) blocks until the compositor says something;
[`dispatch_pending`](Self::dispatch_pending) does not, for a host that also
has its own timer or frame source.
