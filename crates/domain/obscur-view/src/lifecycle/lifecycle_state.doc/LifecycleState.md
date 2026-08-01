The lifecycle phase of a component instance (spec §2.1).

A component moves `Created` → `Mounted` → (`Updating` ⇄ `Mounted`)* →
`Unmounted`. The phase is plain data: it records *where* an instance is in
its life, and the runtime (later sub-steps) drives the transitions. There is
no reactive engine here, so this type only names the states and offers a few
predicates the lifecycle contexts and the error boundary consult.
