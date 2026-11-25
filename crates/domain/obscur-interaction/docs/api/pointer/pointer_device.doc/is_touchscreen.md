Returns whether this device is a touchscreen contact ([`Touch`](Self::Touch)).

Distinct from [`is_direct`](Self::is_direct), which a pen shares: §2.4
treats touch as its own *source* — *"Touch does not synthesize persistent
hover"* — so the two cases separate wherever a contact and a stylus behave
differently, and this names that one device rather than the class.
