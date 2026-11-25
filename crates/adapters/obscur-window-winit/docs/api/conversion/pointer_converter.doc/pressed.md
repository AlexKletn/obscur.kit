Builds a [`PointerEventKind::Down`] event at `position` for pointer `id`,
naming the `button` that went down and the `held` set **after** the press
(spec §4.7).

The two are not the same fact and neither can be recovered from the other:
`held` is what the hand is holding, `button` is what just moved. A press of the
secondary button while the primary is already down reports both as held, and
only `button` says which of them the press was.

This exists because without it §2.3's trusted flag answers nothing where it
matters. A host had no way to obtain a *trusted* press: the builders were
move/enter/leave only, so a real click came in through `PointerEvent::new`
as **untrusted** — exactly like a `§7.6` injected one. The two kinds
DevTools can inject are `PointerDown`/`PointerUp`, and those were precisely
the two this converter could not produce, so the flag's trusted and
injectable sets were disjoint and it could never tell them apart.
