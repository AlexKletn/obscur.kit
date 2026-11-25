The replaceable port for a navigation history stack (spec §6.13 history).

A history provider owns the ordered stack of visited entries and a cursor
marking the current one. The domain drives navigation through this trait and
stores it as `Box<dyn HistoryProvider>`, so it is deliberately **object-safe**
— no generic methods and no `Self`-returning methods. The
[`std::fmt::Debug`] supertrait lets a boxed provider be `Debug` (so the
composing `Router` can derive it).

Mutating navigation takes `&mut self`; queries take `&self`. The colocated
[`MemoryHistory`](super::MemoryHistory) is the reference in-memory implementation and documents the
contract of each method concretely.
