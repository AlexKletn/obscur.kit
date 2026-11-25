Lowers one parsed declaration into a [`CompiledDeclaration`].

The compiler is stateless; the string interner it needs is threaded in per
call so a single instance may lower any number of declarations.
