Whether any of these layers carries a colour matrix — the question that
decides whether the [`ColorMatrix`] pipeline is built at all, so a scene
with only blurs stays byte-identical to the pre-colour-op path.
