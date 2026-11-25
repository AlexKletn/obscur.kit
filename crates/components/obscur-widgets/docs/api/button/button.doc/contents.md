The children a button with this label renders: one text run, or nothing at
all when the label is empty.

One function with two callers, which is the point. [`Button::view`] installs
it under its own root, and [`ButtonComponent`] renders it as the **fallback**
of the default-slot outlet — the content a mounted `<button label="Save"/>`
shows when the author supplied none. A built button and a mounted one are
therefore the same tree by construction rather than by agreement.

An empty label is no child rather than an empty one: the two paint the same,
and it is what keeps an authored `<button on:click=…/>` childless when the
built-ins are registered.

[`ButtonComponent`]: super::ButtonComponent
