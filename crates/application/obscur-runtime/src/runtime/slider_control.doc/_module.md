What a press or an arrow key means to a slider.

The arithmetic that turns a pointer position into a value, and a value into the
pair of ends a ranged control holds. It lives here rather than in the widget
because the RUNTIME is what receives the press — and it reads the scale off the
node through `SliderScale`, the same type the widget draws with, so the value
under the pointer is the value the thumb is drawn at.

Pure: it decides nothing about dispatch and touches no state. Everything it
answers is a function of the node, its box and a point.
