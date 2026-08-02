The component's own render holds an outlet for a slot its contract does not
declare. Nothing will ever be assigned to that name, so the outlet renders its
fallback if it has one and vanishes if it does not.

This is a §2.13 defect as much as a §2.12 one: a contract is the component's
exported description of itself, and one that omits a slot the component renders
is a description that lies.
