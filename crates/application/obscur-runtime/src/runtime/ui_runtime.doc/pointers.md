Every live pointer's interaction state: its outstanding press and click
run, its capture, and the pan, hold or coast it is driving (spec §2.4,
§6.16).

One field rather than the eight it replaces, because a pointer's press,
pan, hold, capture and drag-selection are one *sequence* and have to end
together — see [`PointerSessions`], which owns that teardown.
