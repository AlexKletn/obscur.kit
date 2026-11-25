What a presentation feedback carries with it until the compositor answers.

The sequence number alone was not enough. `discarded` has one meaning in the
protocol and two causes worth telling apart — the surface was not composited, or
a later commit superseded it — and the only thing that separates them is whether
this process committed on the surface in between. So the commit count at the
moment of asking travels with the request and comes back with the answer.
