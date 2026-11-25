The identity by which a monitor is recognized across enumerations (spec
§6.6).

winit exposes no stable monitor identifier — only a name, which is the
connector on X11/Wayland and the adapter path on Windows (unique in both),
but which macOS repeats for two identical panels. The ordinal disambiguates
those by enumeration order.

Geometry is deliberately **not** part of the key: a monitor whose mode,
scale, or position changed is the same monitor and must keep its id, or
every reconfiguration would report as a disconnect followed by a connect.
