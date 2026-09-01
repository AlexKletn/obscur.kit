Pointer input passes straight through to whatever is behind the surface.

Clicks, hovers, scrolls and touches all land on the window or the desktop
underneath, which never learns that the surface is there. The surface still
draws — this says nothing about what is visible, only about what is clickable —
so nothing on it can be hovered, pressed or scrolled, and a surface with a
control on it should not ask for this.
