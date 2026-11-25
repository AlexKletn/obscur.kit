Returns `element`'s slot assignment, if it carries one.

Only a string literal is a slot *name*; `slot={expr}` is a name that does not
exist until runtime, and the rule says so rather than this front end deciding
it alone.
