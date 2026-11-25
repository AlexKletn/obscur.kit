The `background-*` box properties and the `background-image` layer list
(CSS Backgrounds and Borders 3 §3, CSS Images 3 §5).

`background-size`, `-position`, `-repeat` and `object-fit` / `object-position`
are four small keyword-or-length grammars that share one shape — an
axis pair with keyword shorthands — while `background-image` is the crate's
only comma-separated *layer* list, where each layer is a URL or one of the
gradient functions in [`super::gradient_lowering`]. The `background`
shorthand sits here too because what it does is choose between those two
longhands (an image, or a colour) rather than fan out into four.
