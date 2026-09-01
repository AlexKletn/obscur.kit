The other side of the same guard: a name no machine has registers nothing at
all, and the chain falls through to the generic behind it.

Without this, "resolve named families through the host" would quietly become
"resolve every named family to the system default", which is a worse bug than
the one being fixed — an author's fallback chain would never be walked.
