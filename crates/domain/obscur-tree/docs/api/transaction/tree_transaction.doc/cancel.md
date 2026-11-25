Discards the transaction without touching any tree.

This is the **provisional-cancel** path. Because staging never mutates
the live tree, cancelling (or dropping) a transaction is guaranteed to
leave every tree completely unmodified. Provided explicitly so intent is
clear at call sites.
