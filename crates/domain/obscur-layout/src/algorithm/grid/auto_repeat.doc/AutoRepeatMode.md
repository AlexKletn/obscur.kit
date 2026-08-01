Which auto-repeat mode a [`AutoRepeat`] pattern uses (CSS §4.1).

Both modes compute the *same* repetition count from the container content
extent (as many whole copies of the pattern as fit); they differ only in what
happens to a generated track that ends up with no placed item:

* [`AutoFill`](AutoRepeatMode::AutoFill) — empty tracks are **kept**; they
  occupy their sizing-function's space just like any other track.
* [`AutoFit`](AutoRepeatMode::AutoFit) — empty tracks **collapse** to `0` after
  placement and the gaps around them collapse too, so the occupied tracks pack
  together.
