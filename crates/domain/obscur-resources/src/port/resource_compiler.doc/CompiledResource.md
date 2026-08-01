A descriptor of a backend/GPU artifact compiled from a resource (spec §5.15).

This is the output of [`ResourceCompiler::compile`]. The compiled artifact
(texture, pipeline, uploaded atlas) lives inside the adapter that produced it;
the domain keeps only this opaque descriptor, recording the artifact's
backend/GPU `byte_size` — the footprint used for backend-cache budgeting (spec
§5.15) — alongside the [`ResourceMetadata`] carried through the pipeline.
