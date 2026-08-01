Why a GPU device was lost (spec §5.2).

Backends must handle device loss explicitly; this enumerates the causes a
backend may report so the runtime can decide whether to rebuild and retry.
