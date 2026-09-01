Warnings in the log whose location falls under one of the watched paths.

Pairs each `warning:` line with the **first** `-->` that follows it — a rustc
block repeats the arrow for its notes, and only the first names the warning.

Matching is by **substring**, not by prefix, because the form of the path is not
under this check's control: cargo reports a path dependency relative to the
fixture (`../../crates/…`) or absolutely (`/home/…/crates/…`) depending on how
it was invoked, and both were observed while this was being written. The watched
entry is the repository path either form contains.
