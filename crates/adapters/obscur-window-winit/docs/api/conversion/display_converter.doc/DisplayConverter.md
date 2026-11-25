A stateless converter from a winit monitor to the domain's [`Display`]
(spec §6.6, §2.7).

This is a **pure** projection of a monitor snapshot: the caller supplies the
stable [`DisplayId`] (the adapter mints and retains it), and the converter
reads the monitor's current size, refresh rate, scale factor, name, and
position into the domain model.

Global-coordinate honesty (spec §6.6: "On Wayland, *unavailable* global
coordinates are represented honestly"): the condition is unavailability, not a
platform. Wayland compositors do publish each output's logical position, over
**xdg-output**, and winit reports it — so a Wayland session is not by itself a
reason to blank anything. What cannot be read is a compositor that withholds by
answering `(0, 0)` for every output: that is an absence, not an origin, and
publishing it would describe a topology in which every monitor overlaps. The
converter cannot detect that itself, so
the caller chooses the constructor: [`DisplayConverter::new`] projects the
position into bounds, and [`DisplayConverter::without_global_coordinates`]
leaves both bounds `None`, which is what
[`Display::has_global_coordinates`] is there to report.
