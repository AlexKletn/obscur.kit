A mask clip that modulates coverage from separate source content (spec §5.8).

A `Mask` records only *how* its coverage should be interpreted — from the
source's alpha or its luminance (see [`MaskKind`]). It does **not** carry the
source itself: the mask's source content — a rendered sub-scene or image that
provides the coverage field — is supplied by the layer or command that
applies the mask, and is attached at that use site (a forward reference to the
scene/command model). Because the coverage generally comes from rendered
content, applying a mask may require an offscreen pass.
