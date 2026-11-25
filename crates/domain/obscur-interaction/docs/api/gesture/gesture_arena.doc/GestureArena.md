Resolves competing recognizers contending for a single pointer sequence
(spec §6.16).

Multiple [`GestureContender`]s may claim the same pointer sequence. The
arena tracks each contender alongside its current [`RecognizerState`] and
applies the arena rule to award the sequence to a single winner:

- If any contender is [`Recognized`](RecognizerState::Recognized), the
  winner is the recognized contender with the **highest priority**. A tie in
  priority is broken toward the **most recently entered** contender: a
  system gesture is entered into the arena after the application's own
  recognizers, so "System gestures have priority by default" (§6.16) holds
  with no explicit priority granted to either side. System gestures may also
  carry a high priority to win outright.
- Otherwise, if there is at least one contender and every contender is
  terminal but not recognized ([`Failed`](RecognizerState::Failed) /
  [`Cancelled`](RecognizerState::Cancelled)), the result is
  [`NoWinner`](GestureResolution::NoWinner).
- Otherwise the arena is still
  [`Pending`](GestureResolution::Pending).
