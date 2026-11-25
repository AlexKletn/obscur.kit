Checks `element` and its element descendants, accumulating every violation
into `error` rather than stopping at the first — the same
report-them-all policy `Compiler::compile` follows.

Three rules run here, and all three are the compiler's own: the §2.12/§2.13
contract ([`check_contract`]), the §2.11 collection rule
([`check_collection`]), which honours both `<For>` and `<VirtualFor>`, and the
§2.12 outlet rule ([`check_outlet`]), which polices an authored
`<slot name="…"/>`. The
macro adds the [`Span`], which is the whole reason this front end reports
better than the file one — the author's build stops on the tag they wrote.
