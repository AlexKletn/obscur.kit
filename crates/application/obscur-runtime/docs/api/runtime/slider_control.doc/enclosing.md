The slider a node is inside, if it is inside one.

Walks UP: a press lands on whatever the hit test found, which for a slider is
usually the canvas inside it rather than the control. The nearest ancestor that
declares a slider's `low` IS the slider.
