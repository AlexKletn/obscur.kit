Builds the counter UI: a [`Container`] grouping a text label and the two
buttons' element nodes, returned as a single-root [`View`].

A widget exposes its desired output as a [`View`]; to nest a button inside a
container we need its [`ViewNode`]s, which we pull out of the button's root
fragment.
