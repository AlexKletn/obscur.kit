Recognizes a [`Pan`](GestureKind::Pan) — a one-finger drag — from the stream
of positions a single pointer reports (spec §6.16).

A pan is not "the pointer moved". Every real contact jitters by a pixel or
two while a finger presses and lifts, so a recognizer that began on the first
move would turn every tap into a drag and every list would scroll a hair
whenever it was touched. The recognizer therefore stays
[`Possible`](RecognizerState::Possible) until the contact has travelled
further than `slop` from where it started, and only then does it
[`begin`](RecognizerState::Began) — the standard touch-slop rule. Once begun
it reports the delta of each subsequent sample, so the consumer moves content
one-to-one with the finger.

The slop is passed in rather than fixed here: it is a **policy** owned by
whoever knows the display and the input device, not by this type.

Time is data, like everywhere else in this crate — the recognizer reads no
clock, it is told the logical timestamp of each sample.

# Lifecycle

`Possible` → `Began` (slop crossed) → `Recognized` (the contact lifted) or
`Cancelled` (the system, or a competing recognizer, took the sequence). A
contact that lifts without ever crossing the slop
[`Fails`](RecognizerState::Failed): it was a tap, and saying so is what lets
the consumer treat it as one.
