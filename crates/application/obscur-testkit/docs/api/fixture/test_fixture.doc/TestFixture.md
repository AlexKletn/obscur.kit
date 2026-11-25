The one runtime identity and the one viewport the whole harness renders into.

Nothing about a 300×300 viewport is special, but *everything* about every test
using the **same** one is: the fixtures below place their boxes at absolute
coordinates and the tests press at absolute points, so a viewport that differed
between two fixtures would silently move the target out from under a press. The
same holds for the [`RuntimeId`](Self::RUNTIME_ID) — a test that asserts on
identity is asserting on this constant.

The type carries no state. It exists so the two constants and the two
constructors have one name to hang from, rather than being loose functions a
reader has to recognise.
