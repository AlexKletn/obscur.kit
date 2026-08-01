Returns the first dependency cycle found (as an ordered path of crate
names, closing back on the first), or `None` if the graph is acyclic.
Only edges between known workspace nodes are followed.
