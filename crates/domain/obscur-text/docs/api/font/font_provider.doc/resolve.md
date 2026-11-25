Resolves `family` and `attributes` to the best available concrete face,
or `None` when the family is unknown to this provider (spec §5.18).

"Best" means the available face for the family whose attributes are
nearest the request by [`FontAttributes::distance`].
