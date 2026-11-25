Returns `n % modulus` when `n` is a non-negative integer value, else `None`.

TR35's range relations enumerate integers, so a non-integral `n` matches no
range and the caller must treat it as a miss rather than rounding into one.
