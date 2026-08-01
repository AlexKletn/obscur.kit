Returns `true` for any *positioned* scheme — everything except
[`Static`](Self::Static).

A positioned node **establishes a containing block**: its own padding box
becomes the containing block against which its
[`Absolute`](Self::Absolute) descendants resolve their insets.
