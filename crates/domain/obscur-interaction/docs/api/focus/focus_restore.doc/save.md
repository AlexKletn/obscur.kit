Pushes a frame recording what was focused as the current scope opened.

`None` is a real answer, not a missing one: a trap can open while nothing is
focused at all — the first dialog of a freshly started application is the common
case — and that frame must still exist, or the close that follows would pop the
*enclosing* scope's saved node and hand focus to a control the user never left.
