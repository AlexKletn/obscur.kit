Builds a closed menu from `ids` looked up in `registry`, in the given
order (spec §6.7).

This is the kit-rendered menu surface of the unified action model: each
item is derived from the registered [`Command`] via
[`MenuItem::from_command`], so titles and enabled-ness come from the one
descriptor rather than being restated here.

An id is **omitted** from the menu when it is not registered (there is no
title to show and no state to honour) or when its state is not
[`visible`](CommandState::is_visible) — `visible` is what hides an action
from its surfaces, while [`enabled`](CommandState::is_enabled) renders it
greyed and inert.
