The hexagonal port for compiling a CPU representation into a backend/GPU
artifact (spec §5.15 compile, backend/GPU cache).

An adapter implements this once per backend, and the domain stores it as
`Box<dyn ResourceCompiler>`; the trait is therefore deliberately
**object-safe** — no generic methods and no `Self`-returning methods — so
compilers can be held polymorphically. A compiler reports failure through
[`ResourceError`] rather than panicking.
