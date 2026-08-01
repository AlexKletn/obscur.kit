§3.2 UI transactions — "provisional systems such as DnD may use commit/cancel
semantics;" and "heavyweight universal rollback is not required." (making
concrete §9.1 "- application-wide undo/redo;")

NG-UNDO-01: cancelling a provisional drag transaction must not revert an
unrelated, already-committed edit.
