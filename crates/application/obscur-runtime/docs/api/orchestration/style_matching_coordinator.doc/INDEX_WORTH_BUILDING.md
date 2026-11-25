How many nodes a rematch must touch before building a [`RuleIndex`] for it
pays for itself.

The index costs one linear pass over every rule of every sheet to build and
saves a fraction of one such pass per node read. A full restyle reads it two
hundred times and the build vanishes; a hover that dirties three nodes would
pay for a structure it barely opens. The number is a break-even, measured on
the keyboard scene with `style_bench` (a ~130-rule set), not a tuning knob:
either side of it is correct, only one side is faster.
