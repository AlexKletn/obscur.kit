A component that renders `"<label>:<seed>"`, where `seed` is unique to this
*instance*. A remount produces a new seed, so the rendered text is a direct
probe of "was this component's local state preserved?".
