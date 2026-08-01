§7.11: "Optional `PowerProfile` may influence:" read with README core
principle 7: "**Recoverable capability gaps.** Unsupported platform
operations return typed results instead of panicking."

§7.11 marks the whole feature optional, so the requirement that survives on a
platform without it is exactly how absence is reported. The query must return
a typed absence — not a panic, and not a fabricated reading indistinguishable
from a real one.
