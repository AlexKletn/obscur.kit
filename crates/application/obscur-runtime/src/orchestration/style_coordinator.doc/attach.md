Attaches `metrics` to `ctx` when the caller supplied any.

Both walks rebuild their context on reaching the root element (to swap in
the real `rem` base), and a rebuild that forgot the port would silently
un-resolve `ch` for the whole document *below the root* while the root
itself stayed correct — a bug that hides in exactly the tree shape tests
use. Funnelling every construction through here is what stops that.
