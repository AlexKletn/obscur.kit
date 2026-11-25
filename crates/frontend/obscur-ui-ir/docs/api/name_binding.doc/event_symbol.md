Resolves an event name to the [`InternedSymbol`] it binds to.

`click`/`input`/`change` map to the reserved `EVENT_*` symbols the runtime
dispatcher resolves; any other name is hashed like an attribute.
