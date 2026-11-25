Whether the pointer can reach a layer surface at all.

The keyboard half of this question is
[`LayerFocusPolicy`](crate::layer::LayerFocusPolicy); this is the other half,
and a surface answers both. The default is
[`Interactive`](LayerPointerPolicy::Interactive), because a panel nobody can
click is a deliberate choice rather than a starting point.

[`Transparent`](LayerPointerPolicy::Transparent) is what a surface that only
*shows* something wants — a toast, a notification, a heads-up indicator, a
wallpaper. Such a surface is usually up for as long as its process is, showing
nothing most of that time, and every pixel it covers is a pixel of the desktop
that stops responding to a click. Transparency is what keeps it from taking
something it never uses.

A backend that cannot express it reports so through
[`LayerFeature::InputTransparency`](crate::port::LayerFeature::InputTransparency);
the setting is never silently dropped.
