Whether a crate in `self` is allowed to depend on a crate in `target`,
per the allowed dependency directions of spec §2.11 / §5.9.

Note: the facade (`obscur-kit`) is the apex of the graph and the ban on
non-facade crates depending on it is enforced separately, so no layer
here lists `Facade` as an allowed target.
