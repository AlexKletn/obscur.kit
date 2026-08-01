The CSS pipeline, alive for the app's lifetime because it owns the §4.12 parse
cache: a replacement through [`AppControl::set_stylesheets`] re-parses only what
changed identity, and a fresh pipeline per install would throw that away.
