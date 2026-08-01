Returns whether resolving this source requires a host loader outside the
kit.

True for [`ResourceSource::Url`] and [`ResourceSource::Document`]: the kit
performs no network access and reads no host documents by itself (spec
§5.15), so these must be handed to a user/host loader. All other kinds are
either resolvable within the kit or by the loader port from a plain path.
