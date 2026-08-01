Whether `method` is a standard in-place mutator — a method whose whole purpose
is to change its receiver (spec §2.10, "arbitrary mutation").

This is a denylist of the common `Vec`/`String`/collection mutators, not a
proof of purity: a method absent from it is admitted (the method-call gap). It
names only methods that mutate *by convention* in the standard library, so a
pure method that returns a new value — `str::replace`, `Vec::iter` — is not on
it and stays allowed.
