Whether a component strictly requires a capability or merely benefits from
it (spec §2.13).

This is an independent trait of a capability dependency, so it lives in its
own type. `Required` means the component cannot function without the
capability; `Optional` means the component adapts gracefully when it is
absent.
