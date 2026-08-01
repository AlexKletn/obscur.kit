Creates a headless host with the default [`HeadlessConfig`].

The host starts with a fresh runtime (empty tree and scene), a clock at
time zero, an idle executor already installed as the runtime's blocking
port, and a virtual window sized to the configuration's viewport. The
first produced frame will be [`FrameId::FIRST`].
