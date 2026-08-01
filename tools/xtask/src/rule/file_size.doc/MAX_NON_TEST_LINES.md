The most lines of **non-test** code a source file may hold.

Non-test, because spec §3.1 admits per file "одну основную публичную или
доменно значимую сущность, её `impl`, приватные локальные функции без
самостоятельной семантики" **and that entity's unit tests**. The tests are
therefore not optional passengers a file could be relieved of — the spec
*requires* them to live there, most of them reach private items and could
not move, and counting them would push a well-tested entity over the bar for
being well tested. So this rule measures everything above the trailing
`#[cfg(test)] mod tests {` and nothing below it; see [`non_test_lines`].

The number is the owner's operational bar for the file-size campaign that
followed the runtime decomposition, and the workspace it governs meets it:
of 1,829 `.rs` files exactly one is above it, and the campaign's twelve
file-size stages took the runners-up down from 7,272, 4,467, 4,281, 2,835,
2,724, 1,911, 1,672, 1,626, 1,167, 1,077, 1,052 and 1,039 lines. Behind the
one exception the field is close: the next four files are 986, 984, 938 and
906 lines. The ledger below is the whole of the exception surface, and it is
expected to shrink.
