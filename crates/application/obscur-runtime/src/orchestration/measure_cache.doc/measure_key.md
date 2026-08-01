The exact cache key for `measure(node, constraints)`.

The **percentage basis** is part of the key because it is part of the answer: the
same node, offered the same space, resolves `height: 50%` to a different size
under a different containing block — and since `FlowLayout` states a basis when
its own block size is definite (css-sizing-3 §5.1), two calls really can differ
in nothing else. A key that ignored it would serve one containing block's answer
for another's. `NaN` stands in for "no basis stated"; its bit pattern is a
constant, so two unstated bases compare equal here even though `NaN != NaN`.
