The replaceable port for a system clipboard (spec §6.20 data transfer —
clipboard).

Every clipboard operation the domain performs funnels through this trait's
single [`execute`](ClipboardProvider::execute) entry point, which dispatches
copy/cut/paste/clear and reports an honest [`ClipboardResult`]. The trait is
deliberately **object-safe** — no generic or `Self`-returning methods — so
adapters can be held polymorphically behind `Box<dyn ClipboardProvider>`.
