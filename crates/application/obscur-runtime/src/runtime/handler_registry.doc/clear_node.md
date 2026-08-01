Drops every handler **and** every listener at `node`, returning how many
registrations went, counted per listener rather than per key.

One operation, not two, because the two tables are the same node's
bindings: a caller that cleared only one would leave the node half-bound.
