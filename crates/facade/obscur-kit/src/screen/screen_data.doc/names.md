Every bound name, sorted.

This is what [`OkuiScreen::compile_with`](super::OkuiScreen::compile_with)
**declares to the compiler**, so a screen may reference these names and
referencing anything else is a §2.13 compile error rather than a silently
missing value. Sorted so the declaration — and therefore any diagnostic order
— does not depend on hash iteration order.
