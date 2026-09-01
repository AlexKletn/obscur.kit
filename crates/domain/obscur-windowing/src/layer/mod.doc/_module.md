Layer surfaces: panels, docks, bars, wallpapers and overlays — surfaces that
are part of the desktop itself rather than application windows.

**Not in the spec.** §6 describes application windows and says nothing about
desktop surfaces, so nothing in this module derives from it and no section
number is cited. It is an extension, and the shape of it is answerable to the
platforms that have to carry it rather than to a paragraph.

Everything here is platform-neutral by construction: no protocol vocabulary, no
wire encoding, no sentinel values. A backend translates the settings and
reports what it could not honour through
[`LayerFeature`](crate::port::LayerFeature), because the platforms differ in
what they can do and the difference is worth stating rather than hiding.
