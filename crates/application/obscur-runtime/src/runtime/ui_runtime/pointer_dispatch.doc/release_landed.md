Turns an [`Up`](obscur_interaction::pointer::PointerEventKind::Up) into what it
means: a click on the node the press landed on, the activation that click
performs, the virtualized row it resolved to, and the end of everything the
contact was carrying. Returns §2.3's `(command, current_target)` — the resolved
activation command and the node whose binding resolved it — and reports through
`press_changed` whether the `:active` styling state changed.

`prevented` is a `pointerup` listener's refusal. It suppresses the *meaning* of
the release — the click, the activation, the virtualized row click — and none of
its bookkeeping. That split is the point: a refusal declines an action, and
ending the press, the capture and the pan is not an action but the truth that
the finger lifted. Suppressing those would leave a stuck `:active` and a capture
nothing ever releases.

The four things it ends — the press, the hold, the pan and the drag-selection —
end in one call to [`PointerSessions::end_pointer`], so they cannot drift apart;
a pan still travelling when the contact lifted comes back out of it to be
thrown.

[`PointerSessions::end_pointer`]: crate::runtime::PointerSessions::end_pointer
