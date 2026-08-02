The element names the UA sheet styles that no widget and no primitive produces.

This is the guard's only declared vocabulary, and it is deliberately two names
long. `a` and `fieldset` are HTML element names with no counterpart in the
widget or primitive tier; they reach the tree through hand-written markup,
where `name_binding::local_name_for` makes every element tag its own local
name. `the_markup_only_vocabulary_really_reaches_the_tree` proves each entry,
so an addition here has to be an element that renders — but it is still a
declaration, which is why it is short and why adding to it should be argued.
