The two reserved structural states an interaction path has to consult before
it does anything to a node — `disabled` and `readonly` — each answered for the
node **and its element ancestors**.

A borrow bundle over the retained tree, held for one question and released
with it. It reads the tree; it changes nothing and schedules nothing.

# Why the two are one entity

Because the whole content of each answer is *how far it reaches*, and the two
reaches are different in a way that is easy to get backwards. Both states are
carried by an element and inherited by everything inside it — a press on a
label inside a disabled button is a press on a disabled control — so both are
ancestor walks rather than node lookups. What differs is **which paths gate on
them**:

- `disabled` gates interaction *wholesale*. A press seats no focus, no press
  anchor and no editing caret; typing never reaches it; a click / `Enter` /
  `Space` fires no activation command; a long press does not arm. `:hover` is
  deliberately unaffected — CSS still hovers a disabled element — so only the
  **behavioural** paths consult it, never the styling ones.
- `readonly` gates *only mutation*. The control is still focusable and its
  text still selectable and copyable: a press seats focus, a caret and
  drag-selection exactly as on a normal field. Only its **value** is immutable
  (insert / delete / backspace, IME commit, paste, a multiline `Enter`
  newline are dropped).

Written apart, the pair reads as two spellings of the same idea and the
distinction survives only in whichever call site happens to be read first.
Written together, "disabled blocks the interaction, readonly blocks only the
write" is the entity.
