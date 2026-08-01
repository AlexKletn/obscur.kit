`font-family` and `font-weight` lowering (CSS Fonts 4 §2.2, §2.4).

Both are grammars the generic value path would get wrong rather than merely
fail on: a family list is comma-separated *groups* of idents that join into
one name (`Helvetica Neue`), and a weight is a number, a keyword on the
100-900 scale, or one of the two relative keywords the domain keeps as
reserved handles because they cannot be resolved without the parent's
computed weight.
