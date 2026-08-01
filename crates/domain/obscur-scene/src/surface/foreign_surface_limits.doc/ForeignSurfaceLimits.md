The explicit restrictions a foreign surface imposes on scene operations
(Obscur Kit 2 specs §5.5).

Each field records whether a given operation is **allowed** for the surface.
Per §5.5 these capabilities are explicit — they are never silently assumed:
a texture-backed surface is typically [`unrestricted`](Self::unrestricted)
(all operations allowed), while a native child surface/window restricts some
of them (see [`native_child`](Self::native_child)).
