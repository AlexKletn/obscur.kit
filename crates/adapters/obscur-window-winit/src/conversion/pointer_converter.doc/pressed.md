Builds a [`PointerEventKind::Down`] event at `position` for pointer `id`,
with `buttons` held **after** the press (spec §4.7).

This exists because without it §2.3's trusted flag answers nothing where it
matters. A host had no way to obtain a *trusted* press: the builders were
move/enter/leave only, so a real click came in through `PointerEvent::new`
as **untrusted** — exactly like a `§7.6` injected one. The two kinds
DevTools can inject are `PointerDown`/`PointerUp`, and those were precisely
the two this converter could not produce, so the flag's trusted and
injectable sets were disjoint and it could never tell them apart.
