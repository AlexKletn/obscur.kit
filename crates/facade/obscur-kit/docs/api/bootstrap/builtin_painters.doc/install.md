Registers the built-in painters on `runtime`.

Cannot fail and cannot clash: a painter is keyed rather than claimed, so a
later registration under the same key replaces the earlier one. Installing
twice is therefore a re-parameterisation, and an application that registers
its own painter under a built-in's key after this call wins.
