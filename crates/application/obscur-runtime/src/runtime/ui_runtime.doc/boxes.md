The absolute layout boxes retained from the last produced frame, in
**content space** (scroll not folded in). Paint/compose and hit-test read
this field directly and apply each scroll container's offset themselves.
