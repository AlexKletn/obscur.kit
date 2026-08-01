The lifecycle state of a [`GestureRecognizer`] (spec §6.16).

Recognition proceeds `Possible` → `Began` → `Recognized`, or short-circuits
to `Failed` / `Cancelled`. A recognizer may also recognize directly from
`Possible` (discrete gestures such as taps). The three terminal states
([`Recognized`](Self::Recognized), [`Failed`](Self::Failed),
[`Cancelled`](Self::Cancelled)) are **sticky**: once reached, the state no
longer changes.
