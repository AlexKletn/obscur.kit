Whether the focus arrived by keyboard, and so whether `:focus-visible` matches.

A pointer press focuses without it; Tab and any other keyboard move set it. The
runtime decides from the envelope that caused the move — `EventOrigin::Keyboard`
and nothing else — so there is no separate flag for a caller to get wrong.
