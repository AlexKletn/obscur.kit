The widest `children` want when each is offered `offer` of inline space.

The returned width is a **content** width in the parent's content box:
each child's own horizontal margins are subtracted from what it is offered and
added back to what it contributes, so the answer is the space the run needs,
margins included (CSS 2.1 §10.3.3).

Blocks stack, so the widest child decides — not the sum, which is what an
inline or flex row would want.
