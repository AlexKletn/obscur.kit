What a main-axis margin edge costs a flex line (css-flexbox-1 §8.1).

Two functions, and they exist as a pair because an `auto` margin answers the
same question with two different numbers depending on *when* it is asked:
before the line is sized it reserves nothing, after the line is sized it takes
a share of what is left. [`reserved`] and [`used`] are those two moments, and
their doc comments are written against each other — calling the wrong one is
how the free space gets counted twice.

Both are reached from two places (the container's content-main sum and the
line placement pass), which is what makes them an entity of their own rather
than a private helper of either.
