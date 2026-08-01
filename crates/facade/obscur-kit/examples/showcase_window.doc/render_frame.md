Renders one frame: run a scheduler-gated reactive frame through the
runtime (rebuilding the view from the current state when one is due), read
the committed scene, package it, and present it on-screen via Vello.
