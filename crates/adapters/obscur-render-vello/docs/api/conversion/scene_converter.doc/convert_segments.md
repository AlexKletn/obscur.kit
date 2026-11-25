Converts backdrop filters and extractable filter layers into ordered segments.

A backdrop filter reads the frame accumulated up to its command. Ordinary
filter output is composited after preceding draws and before subsequent draws.
Each content segment has its own Vello scene.

A backdrop boundary closes the active clips, transforms and neutral layers in
the preceding content run and restores them in the next one. The coverage
scene rasterizes the transformed panel outline inside the active clips.
Layers with opacity, isolation, blending, masks or effects remain intact
and any backdrop filter inside them produces a fallback diagnostic.

Ordinary filter boundaries preserve the current transform and only extract
outside clip and layer scopes.
