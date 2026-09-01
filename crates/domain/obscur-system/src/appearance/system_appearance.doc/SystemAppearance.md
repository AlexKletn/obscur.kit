What the desktop says about how an interface should look.

Read once, by an [`AppearanceProvider`](super::AppearanceProvider), and handed
to whoever wants it. Every field is optional for the reason
[`SystemPalette`](super::SystemPalette) gives: sources differ in what they know,
and a value invented to fill a gap cannot be told from one the user set.

Deliberately **a snapshot, not a subscription.** A desktop's theme can change
while an application runs, and nothing here notices; a host that cares reads
again. Watching means a thread or a file descriptor on somebody's event loop,
which is a commitment this type does not make on a caller's behalf.
