A deterministic, scripted [`DialogProvider`] for tests (spec §6.9).

`ScriptedDialogProvider` answers each [`Self::present`] call with the next
result queued by [`Self::push_result`], in FIFO order, falling back to
[`DialogResult::Dismissed`] once the queue is empty. Its [`Self::origin`] is
`"scripted"` and it reports [`SupportLevel::Emulated`], letting tests drive
domain logic through the port without any real platform dialog.
