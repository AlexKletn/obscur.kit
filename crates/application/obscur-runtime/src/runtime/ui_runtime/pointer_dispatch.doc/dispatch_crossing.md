Dispatches one crossing to one node.

The binding is resolved on that node **alone** — no ancestor walk, unlike every
other event on this path. The crossing set is already the whole chain, so a
walk would deliver a container's binding once per descendant that crossed with
it. The command it names still routes through §2.3 and §2.6 like any other.
