Whether this source can report a change, as opposed to answering when asked.

**A separate capability from [`support`](Self::support), and deliberately so.**
The two do not move together: a source can state the entire appearance and be
unable to say when it changes — a settings *file* is exactly that — and the
reverse is just as possible. Folding them into one level would leave a host
either polling something that would have told it, or trusting a notification
that is never coming.

The default is
[`Unavailable`](obscur_capability::SupportLevel::Unavailable): most settings
stores have no signal, so a source that says nothing is taken at its word.
