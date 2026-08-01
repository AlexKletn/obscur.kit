The command's effect on nesting depth: `+1` for a push, `-1` for a pop,
and `0` for the leaf [`PaintCommand::DrawPath`].

Folding this over a stream and requiring the running total to stay
non-negative and end at zero is exactly the balance check a renderer
relies on.
