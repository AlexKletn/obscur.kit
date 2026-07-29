//! The out-of-workspace consumer check: does the toolkit work from the far side
//! of the facade?
//!
//! `checks/facade-consumer` is a crate whose only dependency is `obscur-kit`, so
//! its extern prelude holds `obscur_kit` and nothing else. That is the one place
//! a defect in what the *authoring surfaces emit* can be seen: an `okui!`
//! expansion naming `::obscur_view` compiles in every crate of this workspace and
//! in no application anywhere (audit finding 1-A).
//!
//! It cannot be a workspace member — the facade-dependency rule forbids that, and
//! membership would put the internal crates back in the graph — so `cargo test
//! --workspace` does not reach it and this command exists to.

use std::path::Path;
use std::process::Command;

use crate::violation::Violation;

const RULE: &str = "facade-consumer";

/// Relative path of the fixture's manifest, from the workspace root.
const MANIFEST: &str = "checks/facade-consumer/Cargo.toml";

/// Builds and tests the fixture, reporting a violation if it does not pass.
///
/// Shells out to `cargo` rather than reimplementing anything: the claim under
/// test is "an application's build succeeds", and only a real build can make it.
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
