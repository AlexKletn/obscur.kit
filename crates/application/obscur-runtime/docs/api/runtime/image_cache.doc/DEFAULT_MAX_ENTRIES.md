How many decoded images the cache holds before it starts evicting, with no
host budget installed: 512.

A byte cap alone does not bound the *count*, and the count is what the
runtime's per-URL bookkeeping — the resource handles and referrer attributions
in `ResourceBridge` — is sized by. A million four-byte icons fit inside 128 MiB
and would leave a million handles behind them.

512 is well above what any one screen shows and is the count at which
[`DEFAULT_MAX_BYTES`] runs out for a 256×256 RGBA thumbnail, so the two caps
bite at the same place rather than one of them being decorative.
