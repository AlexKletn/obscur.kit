`overflow: hidden` clips, but the user cannot scroll it.

CSS Overflow 3: `hidden` DOES establish a scroll container — it clips, it has a
scrollport, and it can be scrolled programmatically. What it must not do is answer
the user: no scrollbar, and the wheel passes through to whatever can scroll behind
it. Conflating the two turns every clipping box into a wheel target, so a row that
merely clips a long file name swallows the scroll meant for the list around it.
