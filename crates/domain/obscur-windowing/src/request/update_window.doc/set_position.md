Sets a position change, returning the updated delta.

Pass `Some(point)` to request an explicit position, or `None` to clear
the position to a platform-chosen one. Either way the position field is
marked as changed; leaving it unchanged means not calling this setter.
