Converts a domain [`SemanticRole`] into the closest AccessKit [`Role`]
(spec §7.1, §2.7).

The converter is stateless: it holds no configuration and every mapping is a
pure function of its input, so a single shared instance suffices. The domain
role set is deliberately smaller than AccessKit's, so each mapping picks the
nearest AccessKit control type; a handful are approximations, documented on
[`convert`](Self::convert).
