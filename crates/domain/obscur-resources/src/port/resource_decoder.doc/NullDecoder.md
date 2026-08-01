An honest reference [`ResourceDecoder`] that decodes nothing (spec §5.15).

Real decoders live in adapter crates (image, svg, font); `NullDecoder` is the
honest default a hub uses before any decoder is registered. It reports
[`ResourceDecoder::supports`] `== false` for every [`ResourceKind`] and
returns [`ResourceError::Unsupported`] from [`ResourceDecoder::decode`]. Its
refusals are correct behavior, not stubs waiting to be filled in.
