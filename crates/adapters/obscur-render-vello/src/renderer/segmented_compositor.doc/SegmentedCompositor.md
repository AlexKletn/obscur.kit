Composites an ordered list of [`SceneSegment`]s onto one accumulating frame,
frosting that frame in place at each backdrop boundary (spec §5.2).

`backdrop-filter` frosts the frame **as accumulated so far**, which is a
property of the *order* of the whole list rather than of any one command — so
unlike every other effect this backend draws, it cannot be expressed inside a
single Vello scene. That is the entity: a small state machine over an
accumulating target `T`, where a [`Content`](SceneSegment::Content) segment is
rendered to its own transparent offscreen and composited onto `T`
(premultiplied-over, so segments layer in paint order — no Vello load/no-clear
mode needed), and a [`Backdrop`](SceneSegment::Backdrop) boundary blurs the
pixels of `T` that lie under the panel's rounded border box.

It borrows what it needs from the renderer rather than owning any of it: the
host's device, and the two lazily-built pipeline sets that outlive any one
frame. The display list is converted to segments **before** it is constructed,
because conversion is device-free.
