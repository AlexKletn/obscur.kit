#![doc = include_str!("../docs/api/consumer.doc/_module.md")]

use std::path::Path;
use std::process::Command;

use crate::violation::Violation;

#[doc = include_str!("../docs/api/consumer.doc/ConsumerProfile.md")]
struct ConsumerProfile {
    /// The name a violation is reported under.
    rule: &'static str,
    /// The fixture's manifest, relative to the repository root.
    manifest: &'static str,
    #[doc = include_str!("../docs/api/consumer.doc/watched.md")]
    watched: &'static [&'static str],
}

#[doc = include_str!("../docs/api/consumer.doc/PROFILES.md")]
const PROFILES: [ConsumerProfile; 2] = [
    ConsumerProfile {
        rule: "facade-consumer",
        manifest: "checks/facade-consumer/Cargo.toml",
        watched: &[],
    },
    ConsumerProfile {
        rule: "layer-consumer",
        manifest: "checks/layer-consumer/Cargo.toml",
        watched: &["crates/facade/obscur-kit/src/app/"],
    },
];

/// Run every profile and collect what each found.
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    PROFILES
        .iter()
        .flat_map(|profile| check_profile(root, profile))
        .collect()
}

#[doc = include_str!("../docs/api/consumer.doc/check_profile.md")]
fn check_profile(root: &Path, profile: &ConsumerProfile) -> Vec<Violation> {
    let manifest = root.join(profile.manifest);
    if !manifest.is_file() {
        return vec![Violation::new(
            profile.rule,
            &manifest,
            "the out-of-workspace consumer fixture is missing".to_owned(),
        )];
    }

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = Command::new(cargo)
        .arg("test")
        .arg("--manifest-path")
        .arg(&manifest)
        // Inside the workspace's own `target/`, one directory per profile. The
        // fixtures resolve different features and so cannot share artifacts, but
        // they can share a cache: CI restores `target/` and nothing else, so a
        // fixture building beside its manifest recompiles its whole graph — for
        // the layer profile that is wgpu and vello — on every run.
        .env(
            "CARGO_TARGET_DIR",
            root.join("target/checks").join(profile.rule),
        )
        .output();

    let output = match output {
        Ok(output) => output,
        Err(error) => {
            return vec![Violation::new(
                profile.rule,
                &manifest,
                format!("could not run cargo for the consumer fixture: {error}"),
            )];
        }
    };

    // The compiler's own words, not a summary of them: a resolution error in
    // generated code is the whole finding.
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return vec![Violation::new(
            profile.rule,
            &manifest,
            format!(
                "a crate depending on the facade alone does not build:\n{}",
                stderr.trim_end()
            ),
        )];
    }

    watched_warnings(&stderr, profile.watched)
        .into_iter()
        .map(|warning| {
            Violation::new(
                profile.rule,
                &manifest,
                format!(
                    "building this profile warns about facade code that only this \
                     profile compiles — the warning is the finding:\n{warning}"
                ),
            )
        })
        .collect()
}

#[doc = include_str!("../docs/api/consumer.doc/watched_warnings.md")]
fn watched_warnings(stderr: &str, watched: &[&str]) -> Vec<String> {
    if watched.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut pending: Option<&str> = None;
    for line in stderr.lines() {
        if let Some(message) = line.strip_prefix("warning: ") {
            pending = Some(message);
            continue;
        }
        let trimmed = line.trim_start();
        let Some(location) = trimmed.strip_prefix("--> ") else {
            continue;
        };
        // Substring, not prefix: cargo reports a path dependency relatively
        // (`../../crates/…`) or absolutely depending on how it was invoked, and
        // both forms turned up while this was being written.
        if let Some(message) = pending.take()
            && watched.iter().any(|prefix| location.contains(prefix))
        {
            out.push(format!("warning: {message}\n  --> {location}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const WATCHED: &[&str] = &["crates/facade/obscur-kit/src/app/"];

    /// The exact shape audit finding 1-A left in the build log, and the reason
    /// this scan exists: the compiler had already said it, in a feature set
    /// nobody ran.
    const DEAD_FIELDS: &str = "\
warning: fields `css`, `title`, `app_id`, `size`, and `control` are never read
  --> ../../crates/facade/obscur-kit/src/app/app_parts.rs:40:16
   |
40 | pub(super) struct BootedApp {
";

    #[test]
    fn a_warning_in_the_watched_path_is_a_finding() {
        let found = watched_warnings(DEAD_FIELDS, WATCHED);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("are never read"), "{found:?}");
        assert!(found[0].contains("app_parts.rs"), "{found:?}");
    }

    #[test]
    fn a_warning_anywhere_else_is_not() {
        // The whole workspace carries a warning baseline this check must not
        // adopt; only the code this profile is the sole compiler of counts.
        let elsewhere = "\
warning: struct `Vector` is never constructed
  --> ../../crates/foundation/obscur-geometry/src/primitive/vector.rs:5:19
";
        assert!(watched_warnings(elsewhere, WATCHED).is_empty());
    }

    #[test]
    fn a_profile_that_watches_nothing_reports_nothing() {
        assert!(watched_warnings(DEAD_FIELDS, &[]).is_empty());
    }

    #[test]
    fn each_warning_is_paired_with_its_own_location() {
        let two = format!(
            "{DEAD_FIELDS}\nwarning: unused import: `Size`\n  --> ../../crates/facade/obscur-kit/src/app/layer_runner.rs:3:5\n"
        );
        let found = watched_warnings(&two, WATCHED);
        assert_eq!(found.len(), 2);
        assert!(found[0].contains("app_parts.rs"), "{found:?}");
        assert!(found[1].contains("layer_runner.rs"), "{found:?}");
        assert!(found[1].contains("unused import"), "{found:?}");
    }

    #[test]
    fn a_warnings_own_first_location_is_the_one_taken() {
        // A rustc block repeats `-->` for each note; only the first belongs to
        // the warning, and a second must not manufacture a duplicate finding.
        let with_note = "\
warning: fields are never read
  --> ../../crates/facade/obscur-kit/src/app/app_parts.rs:40:16
   |
   = note: see also
  --> ../../crates/facade/obscur-kit/src/app/app_parts.rs:41:9
";
        assert_eq!(watched_warnings(with_note, WATCHED).len(), 1);
    }

    #[test]
    fn every_profile_names_a_fixture_under_checks() {
        for profile in &PROFILES {
            assert!(profile.manifest.starts_with("checks/"), "{}", profile.rule);
            assert!(
                profile.manifest.ends_with("/Cargo.toml"),
                "{}",
                profile.rule
            );
        }
    }

    #[test]
    fn the_two_profiles_are_two_fixtures() {
        // One profile per fixture, or the second would silently re-check the
        // first and report a gap as covered.
        assert_ne!(PROFILES[0].manifest, PROFILES[1].manifest);
        assert_ne!(PROFILES[0].rule, PROFILES[1].rule);
    }
}
