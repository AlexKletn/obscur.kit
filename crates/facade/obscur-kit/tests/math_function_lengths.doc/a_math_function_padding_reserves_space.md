The same rule one property over, to pin that the fix was the shared component
rule and not a special case for radii: `padding` is space the child starts
after.

The padded box is *inside* the root rather than being it, because the root
fills the viewport whatever it asks for — measuring padding on the root would
measure the viewport instead.
