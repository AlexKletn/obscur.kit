Builds the overlay for a node's absolute border-box `bounds`: a translucent
fill under a solid outline. Both are leaf [`PaintCommand::DrawPath`]s, so
appending them to a balanced display list keeps it balanced.
