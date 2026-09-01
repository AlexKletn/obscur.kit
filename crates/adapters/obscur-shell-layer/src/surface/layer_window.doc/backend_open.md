Re-apply settings to a surface that is already open.

The port opens a surface from settings; this backend's surface is opened by
[`LayerWindow::open`](Self::open), because a Wayland connection and a calloop
loop have to exist before there is a `self` to call a trait method on. So the
port's `open` means *apply* here, and it is not a lie: after it returns, the
surface is living under exactly the settings that were passed.
