The hit region each canvas declared, measured and in layout space.

Kept measured rather than as a raw path because hit testing asks "is this point
inside" on every pointer report, and flattening per question would do the same
work over and over.
