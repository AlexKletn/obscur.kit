An ordered `font-family` list, in author order. Each entry is either a
concrete family name (for example `"Helvetica Neue"`) or a generic
keyword (`"serif"`, `"sans-serif"`, `"monospace"`, `"cursive"`,
`"fantasy"`, `"system-ui"`). Family names are stored **as strings**, not
interned symbols: the runtime matches them against its font registry by
string, so this value crosses the CSS/runtime interner boundary without a
shared interner.
