The lines of every public (`pub fn` / `pub struct` field / `pub const fn`)
item in `crate_dir`'s `src`, paired with their file and 1-based line number.

Test-only code (`#[cfg(test)]` modules) is excluded by dropping everything
from the first `mod tests` marker in a file onward.
