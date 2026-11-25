Collects the three tables for `text` under a finalized `breaks` list, the
`hyphenator` port, and the resolved `white_space` mode.

`breaks` must already be the list the packer will use — the hyphenation
offsets are collected *between* consecutive break opportunities, so a
break the caller later removes (`word-break: keep-all`) would otherwise
leave interior points that split across what is now one word.
