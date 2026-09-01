`Native` where `gdbus` is present.

The portal declares a `SettingChanged` signal and emits it per key — verified by
introspection on a live session. This is the one source here that was designed
to be watched.
