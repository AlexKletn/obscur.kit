The `filter` layer pass: the sub-scenes the converter peeled off the main
scene, rendered into their own offscreens and composited back through the
adapter's owned wgpu passes (spec §5.2).
