Computes the difference from `old` to `new`.

Every property present in either style is compared; one that was added,
removed, or whose computed value differs is recorded. Custom properties are
compared as a whole: the flag is set when any name was added, removed, or
its value changed.
