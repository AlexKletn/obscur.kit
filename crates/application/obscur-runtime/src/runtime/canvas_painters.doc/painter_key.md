Which painter an element is asking for, if any.

The node's own declaration wins. A widget binds its painter as a prop — that is
part of what the widget IS — and a stylesheet rule matching it must not be able
to replace the routine that draws a progress bar with one that draws a clock.
CSS supplies a painter for the elements that declare none, which is the case
`-obscur-painter` exists for.
