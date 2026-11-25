Pins the one property that decides how large this renderer's font table can
get: a registered face is the caller's allocation, not a duplicate of it.

Address equality, deliberately, and not byte equality — byte equality was always
true and was true of the copy this replaces. The whole claim is about how many
copies of a 26 MB face are resident, and only the address can say.

The second half checks that nothing was taken away: a host holding a plain
`Vec<u8>` still registers, and still ends up with those bytes.
