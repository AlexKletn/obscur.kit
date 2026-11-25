What can be learned about the desktop before committing to a surface.

It exists because of an ordering problem with real consequences.
[`LayerSettings::with_display`](obscur_windowing::layer::LayerSettings::with_display)
takes a `DisplayId`, and an id is only meaningful once a backend has enumerated
its outputs — but the settings have to be complete *before* the surface opens.
Without a way to look first, an application could only pass a position in the
compositor's discovery order, which is an implementation detail and a poor thing
to ask a person for.

Left to the compositor, "the output" on a multi-head desktop is whichever one
had focus when the surface was created. For a bar that is untidy; for an
on-screen keyboard it means the keys land on a different screen from the field
being typed into.
