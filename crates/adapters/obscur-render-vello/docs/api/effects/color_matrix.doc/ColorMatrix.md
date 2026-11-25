A cached wgpu pipeline applying a folded 4×5 color matrix in one fragment pass.

Built once via [`ColorMatrix::new`] and reused across frames; each
[`ColorMatrix::apply`] call only allocates the small uniform buffer a single
filter needs (the destination scratch texture is owned by the caller so it can
be fed into a following blur/composite).
