#![doc = include_str!("consumer.doc/_module.md")]

use std::path::Path;
use std::process::Command;

use crate::violation::Violation;

const RULE: &str = "facade-consumer";

#[doc = include_str!("consumer.doc/MANIFEST.md")]
const MANIFEST: &str = "checks/facade-consumer/Cargo.toml";

#[doc = include_str!("consumer.doc/check.md")]
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let manifest = root.join(MANIFEST);
    if !manifest.is_file() {
        return vec![Violation::new(
            RULE,
            &manifest,
            "the out-of-workspace consumer fixture is missing".to_owned(),
        )];
    }

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = Command::new(cargo)
        .arg("test")
        .arg("--manifest-path")
        .arg(&manifest)
        .output();

    match output {
        Ok(output) if output.status.success() => Vec::new(),
        Ok(output) => {
            // The compiler's own words, not a summary of them: a resolution error
            // in generated code is the whole finding.
            let stderr = String::from_utf8_lossy(&output.stderr);
            vec![Violation::new(
                RULE,
                &manifest,
                format!(
                    "a crate depending on the facade alone does not build:\n{}",
                    stderr.trim_end()
                ),
            )]
        }
        Err(error) => vec![Violation::new(
            RULE,
            &manifest,
            format!("could not run cargo for the consumer fixture: {error}"),
        )],
    }
}
