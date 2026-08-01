§5.18 "Default font provider: - system fonts on native platforms; - bundled
fallback set for WASM/isolated environments; … - usable rendering out of the
box."

The isolated case: system font enumeration returns nothing. Text must still
render, from the bundled set.
