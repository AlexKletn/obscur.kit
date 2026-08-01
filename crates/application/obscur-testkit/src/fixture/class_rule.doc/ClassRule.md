A `StyleRule` whose selector is a single class, optionally qualified by one
pseudo-class.

Almost every fixture sheet in this crate is built out of these: the views author
`class="panel"` and the sheet says what a `.panel` is. Spelling that by hand is
five nested constructors (`SelectorList` → `ComplexSelector` →
`CompoundSelector` → `SimpleSelector::Class`) around a `Vec<BlockDeclaration>`,
and the nesting is the same every time — which is exactly the shape that belongs
behind one name.

The declaration methods cover what the fixtures need to say and no more: a box
has a size, it may have a background, and it may scroll. Anything else goes
through [`declaring`](Self::declaring), which takes the raw declarations.
