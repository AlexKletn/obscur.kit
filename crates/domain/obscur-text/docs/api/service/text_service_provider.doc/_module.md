The replaceable port for optional language services (spec §5.13).

This module defines [`TextServiceProvider`](super::TextServiceProvider), the hexagonal port through which
the text domain requests optional language analyses — spellcheck, grammar,
autocorrect, language detection, and user-dictionary lookups (spec §5.13).
Interface adapters wrap a real linguistic engine behind it; the domain never
depends on one, so the honest default ([`NullTextServiceProvider`](super::NullTextServiceProvider)) supplies
nothing and no engine ships in the core package.

The service module provides two reference implementations: [`NullTextServiceProvider`](super::NullTextServiceProvider), the
no-op honest default, and [`MockTextServiceProvider`](super::MockTextServiceProvider), a deterministic
in-memory spellchecker used to exercise the port in tests. Neither is a real
linguistic engine.
