Reserved `placeholder` attribute symbol (`obscur_view::view::PLACEHOLDER`).

The odd one out in this block: every other reserved attribute here is an
input to the *cascade* — a type, a class, a state the matcher reads. This one
is an input to the *painter*. §4.16 makes the placeholder hint a synthetic
part (paint-only, hit-transparent, never a tree node); the runtime takes its
text from this attribute and its style from `input::part(placeholder)`.
Nothing in the pipeline reads a hashed `placeholder` prop, so spelling it any
other way is inert.
