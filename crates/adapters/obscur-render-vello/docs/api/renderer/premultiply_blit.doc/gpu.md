A device and queue, or a panic naming what is missing.

Every test in this module is about a GPU resource — a bind group, and what
reaches a render target through it — so none of them can be answered without a
device. This used to hand back `None` and let each test `eprintln!` and `return`,
which made "no adapter" indistinguishable from "checked and correct": on a
machine or a CI runner with no GPU the module reported three passes having
verified nothing. A test that passes where it checked nothing is worse than an
absent one, because it also buys a false sense of coverage.

So absence is an error here, and the tests carry `#[ignore]` to keep that error
from firing where it is not wanted. The two halves are deliberate:

* `#[ignore]` is what runs by default. `cargo test` on a GPU-less runner reports
  these in the **ignored** column — a number that is visibly not "passed", which
  is the whole point.
* asking for them anyway (`-- --ignored`) is a statement that this machine has a
  GPU. If it does not, the run is broken and says so, rather than printing a
  green line.

The cost is that a developer machine with a working adapter must ask. That is a
named command, written on the `#[ignore]` reason of each test, not folklore.
