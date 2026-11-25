Whether this key came from a real device or was synthesised.

Fails closed: `Untrusted` is what `new` produces, so trust is conferred at the
platform boundary and never assumed. §2.3 gates default actions on it so
injected input cannot silently perform a privileged action.
