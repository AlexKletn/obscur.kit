A gesture recognizer with a lifecycle [`RecognizerState`] (spec §6.16).

A recognizer observes a pointer sequence and drives its state toward a
terminal outcome. Recognizers contend inside a
[`GestureArena`](super::GestureArena); the arena awards the sequence to a
single winner and cancels the rest.

State transitions (terminal states are sticky — no-ops once reached):

- [`begin`](Self::begin): `Possible` → `Began`.
- [`recognize`](Self::recognize): `Possible` or `Began` → `Recognized`.
- [`fail`](Self::fail): any non-terminal → `Failed`.
- [`cancel`](Self::cancel): any non-terminal → `Cancelled`.
