`.counter { background-color: red; height: 50px }` — height and paint but **no
width**.

The omission is the point: with no cascaded width the node's width is driven
purely by the animation under test, so a sampled box width is unambiguous
evidence that the motion overlay reached layout.
