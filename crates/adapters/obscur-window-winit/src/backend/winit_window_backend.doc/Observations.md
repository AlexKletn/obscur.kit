What the probe managed to observe from inside the running loop.

Each field is an `Option` so an assertion cannot pass by never having run:
a turn that never happened leaves `None`, which fails against the
`Some(true)` the test demands.
