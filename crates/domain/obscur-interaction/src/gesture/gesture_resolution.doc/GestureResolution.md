The outcome of resolving a [`GestureArena`](super::GestureArena) (spec
§6.16).

- [`Winner`](Self::Winner): a recognizer won the pointer sequence; the arena
  cancels all other contenders.
- [`NoWinner`](Self::NoWinner): every contender failed — no gesture.
- [`Pending`](Self::Pending): contenders remain active and none has
  resolved yet.
