What a press, a drag and an arrow key do to a slider.

A slider is CONTROLLED, like a text field: the runtime computes the new value
and reports it through the node's `on:input` binding, and the application's
re-render is what moves the thumb. Nothing here writes a value into the tree —
a control that moved itself and then disagreed with the model behind it is the
defect the controlled-value model exists to prevent.
