The current version of every observable source, so [`track`](Self::track)
can record *which* generation a dependency was taken at. Read-only: a
component observes sources, it never advances them (values flow down).
