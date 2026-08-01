§4.9: "Virtualized layout supports:" / "- stable item identity;"

A virtualized list keyed `["b","c","d","e","f"]` has the row for key `"d"`
toggled on. Prepending key `"a"` moves `"d"` from index 2 to index 3. The
row for `"d"` must still be the toggled one and must be backed by the **same
node identity** — recycling keyed by viewport slot or item index would move
the toggle to `"c"`.
