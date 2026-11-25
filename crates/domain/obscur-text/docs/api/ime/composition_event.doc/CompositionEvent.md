A typed IME composition lifecycle event (spec §5.12 start/update/commit/cancel).

These events map 1:1 onto [`Composition`](super::Composition)'s state
transitions: [`Start`](Self::Start) opens a session,
[`Update`](Self::Update) carries a new [`Preedit`], [`Commit`](Self::Commit)
finalizes with the committed text, and [`Cancel`](Self::Cancel) abandons it.
[`Commit`](Self::Commit) and [`Cancel`](Self::Cancel) are the terminal events
(see [`is_terminal`](Self::is_terminal)).

Per §5.12 duplicate keydown/text delivery must be avoided: the driver is
responsible for de-duplicating redundant events (for example a repeated
`Start` or a `Commit` after the session already ended), which the engine then
ignores.
