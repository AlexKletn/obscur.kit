Resolves a [`Clip`] against the enclosing `parent` mask into the effective
clip in force for the commands it encloses.

Shared by [`PaintCommand::PushClip`](obscur_scene::command::PaintCommand::PushClip)
and by a layer's own [`clip`](obscur_scene::layer::SceneLayer::clip) so the two
cannot drift: a layer clip and a clip command naming the same region confine
drawing identically, as Vello resolves both through one rule (spec §5.2 requires
clipping to match across backends).
