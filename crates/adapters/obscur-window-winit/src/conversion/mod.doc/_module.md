Pure winit→domain event converters (spec §6.1 windowing; §2.7 "преобразует
native events в доменные события"). Each converter is a stateless,
deterministic function of its winit input — no event-loop state — so the
§5.2/§6.1 determinism boundary holds.
