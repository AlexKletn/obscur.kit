Splits `args` at top-level [`ParsedValue::Comma`]s (nested function arguments
keep their own commas inside their [`ParsedValue::Function`] node, so they are
never seen here).
