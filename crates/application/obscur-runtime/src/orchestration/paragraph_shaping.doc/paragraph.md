The multi-font orchestrator turning text-node strings into font-tagged
runs. The default composition is deterministic (single run, primary face,
mock glyphs), which is what the cross-backend determinism boundary relies
on. A host swaps its ports via the setters.
