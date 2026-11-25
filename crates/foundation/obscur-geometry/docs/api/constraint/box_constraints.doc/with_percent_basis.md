Returns these constraints with the containing block's content size
attached, so a percentage size resolves against **it** rather than against
whatever range the box was handed.

Set it wherever the caller already knows the containing block — arranging
a child, laying out a root against the viewport. A measure pass asking a
box how small it can be knows no such thing and leaves it unset.
