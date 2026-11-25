Lowers the slot into its `.with_slot(...)` entry on the **typed**
contract.

Child tags are named through [`component_id`], the shared `name_binding`
resolution, so this agrees with `OtherProps::contract().id` **and** with
the `ComponentId` a front end emits for that tag. It used to agree with
only the first: a bare `fnv64` matched the typed model's internal spelling
while `name_binding::component_id_for` resolved every unregistered tag to
`Container`, so a slot's `child = "Tab"` named a handle no authored `<Tab/>`
ever carried. Closing that is the same change that makes `<Card/>` a
component at all.
