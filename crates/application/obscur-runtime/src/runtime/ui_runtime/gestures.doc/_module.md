The runtime's half of the §6.16 touch recognizers: arming a pan and a hold on a
contact, feeding them the moves and the ticks they need, and acting on what they
conclude.

The recognizers themselves are not here. [`GestureArming`] decides *whether* a
contact could pan or hold, from the tree, the computed styles, the boxes, the
scroll offsets and the virtual lists; [`PointerSessions`] owns every armed pan,
hold and fling and answers what each of them has become. What is left for the
runtime is the part only the runtime can do: scrolling a container, firing a
long-press command, requesting the animation frames a countdown and a coast need
to keep arriving, and telling the click path that a recognized gesture has taken
the sequence away from it.

That last one is the reason these five methods are one subject rather than five.
A pan, a hold and a fling are alternatives — travel rules out the wait, the wait
rules out the tap, the throw is what a released pan becomes — and each of them
resolves by *discarding* something another path was about to act on. Reading
them apart from each other is how a flick comes to activate the row it started
on.

[`GestureArming`]: crate::runtime::GestureArming
[`PointerSessions`]: crate::runtime::PointerSessions
