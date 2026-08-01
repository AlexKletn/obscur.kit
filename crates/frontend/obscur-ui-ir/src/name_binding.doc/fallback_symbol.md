Hashes an unreserved name into a high symbol namespace (top bit set) that
never collides with the reserved low range.

This is the deterministic placeholder both frontend paths share for names
that have no reserved symbol; a name interner threaded through the whole
pipeline would replace it.

# A handler name is one of these, and being *claimed* is a separate question

For an `on:<event>` binding this function computes the **command symbol** a
runtime registration is matched against, and it is equal to
`InternedSymbol::named` by construction (pinned by
`okui_name_binding_drift.rs`). What it cannot say is whether anybody registered
that command: hashing `incrment` succeeds exactly as hashing `increment` does,
so a misspelled handler used to be a silent no-op. That check is
`obscur_okui_compiler::validation::CommandValidator`, against a set of command
names the caller declares — because the registrations are runtime values and no
compiler can discover them. The file front end can be handed that set; the
`okui!` macro cannot, for exactly the reason the module docs give for an
unregistered component tag.
