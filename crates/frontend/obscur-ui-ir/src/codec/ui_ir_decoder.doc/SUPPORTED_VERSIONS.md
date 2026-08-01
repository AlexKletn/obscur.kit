The inclusive range of IR schema versions this decoder can read: from
the first shipped version up to [`UiIrVersion::CURRENT`] (spec §2.15). A
document declaring a version outside this range is refused with
[`UiIrDecodeError::UnsupportedIrVersion`].
