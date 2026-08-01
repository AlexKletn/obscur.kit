Drains and returns the window events queued since the last poll, each
paired with the window it targets (spec §6.1-6.5).

[`WindowEvent`] carries no [`WindowId`] of its own, so the pairing is
supplied here. Returns an empty [`Vec`] when nothing is queued.
