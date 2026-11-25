Marks `root` and every node beneath it as pending a restyle.

`out` doubles as the visited set: a node already recorded was recorded by a
previous `mark_subtree`, which by definition also marked its whole subtree,
so skipping it skips work that is provably already done. That also makes the
walk cycle-safe — a malformed child link terminates instead of looping.
