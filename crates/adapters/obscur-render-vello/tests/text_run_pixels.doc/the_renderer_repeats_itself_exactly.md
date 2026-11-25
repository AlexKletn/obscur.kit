The control, and the reason every comparison in this file counts *visible*
pixels rather than bytes.

Six renders of one list, back to back through one renderer, agree on every
pixel that reaches a screen — so a visible difference measured below is the
drawing and not the GPU. They do **not** agree byte for byte: about six bytes
of the 40 960 move between runs, always inside pixels whose alpha is zero on
both sides, where the fine shader's colour channels are simply not defined.
Anything comparing this backend's read-back byte by byte will flake on that;
this test pins the distinction down so the next such test is written knowing it.
