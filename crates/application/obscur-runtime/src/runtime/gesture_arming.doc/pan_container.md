The scroll container a pan should be armed for, when `event` is a
**direct** contact that went down inside one (spec §6.16 `Pan`).

Direct only — touch and pen. A mouse drag inside a list is a text
selection or a data drag, and turning it into a scroll would break both;
the wheel is the mouse's scroll gesture and it already works. A finger has
no wheel, so dragging the content *is* how it scrolls.

Arming on the answer is cheap and speculative: the recognizer starts
`Possible` and stays there unless the finger travels past the slop, so a
tap arms one and nothing else happens.

The container is resolved from the **screen** point — before each
container's own scroll offset is folded in — because the scroll-mapped
point moves as the container scrolls, so tracking it would feed the pan
its own output.
