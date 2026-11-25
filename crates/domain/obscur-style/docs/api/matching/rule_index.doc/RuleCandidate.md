Where one rule sits in the sheet set: which sheet, which rule inside it, and
the global declaration index its block starts at.

The last field is the one that makes skipping rules safe. The unindexed walk
increments a single `source_order` counter over *every* declaration it
encounters, matched or not; carrying the running total per rule reproduces
that number without walking the rules in between.
