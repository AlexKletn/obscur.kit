Lowers `repeating-linear-gradient()` using the same direction syntax as
[`lower_linear_gradient`]. Stops accept lengths, percentages and automatic
positions. A two-position stop produces two stops of the same color.

The period spans the first through the last resolved stop. Stops with mixed
units, an offset first stop, or an automatic endpoint retain their authored
positions until layout supplies the gradient-line length. Origin-aligned
repeats with a known positive period keep their normalized representation.
