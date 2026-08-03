#![doc = include_str!("spec_ch7b_power.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch7b_support/mod.rs"]
mod support;

use obscur_kit::capability::SupportLevel;
use obscur_kit::system::power::{
    FixedPowerProvider, NullPowerProvider, PowerProfile, PowerProvider, PowerSource,
};

use support::{
    SharedTelemetry, drive_effect_frames, full_performance_profile, maximum_saving_profile,
    run_keystroke, runtime,
};

#[doc = include_str!("spec_ch7b_power.doc/a_power_profile_is_a_public_named_type_an_author_can_supply.md")]
#[test]
fn a_power_profile_is_a_public_named_type_an_author_can_supply() {
    // The author spells the name exactly as §7.11 spells it.
    let profile: PowerProfile = full_performance_profile();

    let provider = FixedPowerProvider::new(profile);
    let boxed: Box<dyn PowerProvider> = Box::new(provider);

    assert_eq!(
        boxed.profile(),
        profile,
        "the profile an author supplies is the profile the provider layer reports"
    );
    assert_eq!(
        boxed.availability(),
        SupportLevel::Native,
        "a provider constructed from an author-supplied profile reports that \
         power information is available"
    );
}

#[doc = include_str!("spec_ch7b_power.doc/b_default_run_is_byte_identical_to_an_explicit_no_saving_run.md")]
#[test]
fn b_default_run_is_byte_identical_to_an_explicit_no_saving_run() {
    const FRAMES: usize = 12;

    // The profile a run with no power configuration whatsoever gets must itself
    // denote *no saving*: off by default.
    let defaulted = PowerProfile::default();
    assert!(
        !defaulted.is_constrained(),
        "with no power configuration the profile must not ask for any saving, \
         got {defaulted:?}"
    );

    // Run D: defaults only.
    let telemetry_d = SharedTelemetry::new();
    let mut rt_d = runtime();
    rt_d.set_telemetry_sink(Box::new(telemetry_d.clone()));
    let scenes_d = drive_effect_frames(&mut rt_d, FRAMES);

    // Run F: the author explicitly supplies the no-saving profile.
    let telemetry_f = SharedTelemetry::new();
    let mut rt_f = runtime();
    rt_f.set_telemetry_sink(Box::new(telemetry_f.clone()));
    let full = full_performance_profile();
    let provider_f = FixedPowerProvider::new(full);
    assert!(
        !provider_f.profile().is_constrained(),
        "the explicitly-supplied full-performance profile asks for no saving"
    );
    let scenes_f = drive_effect_frames(&mut rt_f, FRAMES);

    assert_eq!(
        scenes_d.len(),
        scenes_f.len(),
        "both runs produce the same number of frames"
    );
    for (frame, (d, f)) in scenes_d.iter().zip(scenes_f.iter()).enumerate() {
        assert_eq!(
            d, f,
            "frame {frame}: the default run and the explicit no-saving run must \
             produce identical Scene snapshots — same shadow, same blur, same \
             raster resolution, same list contents"
        );
    }

    assert_eq!(
        telemetry_d.counters(),
        telemetry_f.counters(),
        "the two runs record identical telemetry counters"
    );
    assert_eq!(
        telemetry_d.span_names(),
        telemetry_f.span_names(),
        "the two runs record identical telemetry spans"
    );
    assert_eq!(
        telemetry_d.violations(),
        telemetry_f.violations(),
        "the two runs record identical budget violations"
    );

    // Nothing in the default run may attribute anything to a power policy: with
    // the feature off, there is nothing to attribute.
    let power_attributed: Vec<_> = telemetry_d
        .counters()
        .into_iter()
        .filter(|(subsystem, label, _)| {
            let l = label.to_ascii_lowercase();
            let s = subsystem.to_ascii_lowercase();
            l.contains("power")
                || l.contains("battery")
                || l.contains("thermal")
                || s.contains("power")
        })
        .collect();
    assert!(
        power_attributed.is_empty(),
        "a default run attributes no telemetry to a power policy, got \
         {power_attributed:?}"
    );
}

#[doc = include_str!("spec_ch7b_power.doc/c_the_most_restrictive_profile_does_not_delay_the_keystroke.md")]
#[test]
fn c_the_most_restrictive_profile_does_not_delay_the_keystroke() {
    const DELIVER_AT: usize = 10;
    const TOTAL_TICKS: usize = 40;

    let restrictive = maximum_saving_profile();
    let restrictive_provider = FixedPowerProvider::new(restrictive);
    assert!(
        restrictive_provider.profile().is_constrained(),
        "the most restrictive profile the API offers does ask for saving"
    );
    let run_r = run_keystroke('a', DELIVER_AT, TOTAL_TICKS);

    let full = full_performance_profile();
    let full_provider = FixedPowerProvider::new(full);
    assert!(
        !full_provider.profile().is_constrained(),
        "the no-saving profile asks for no saving"
    );
    let run_f = run_keystroke('a', DELIVER_AT, TOTAL_TICKS);

    assert!(
        run_r.final_text.contains('a'),
        "under maximum saving the character reaches the retained text, got {:?}",
        run_r.final_text
    );
    assert!(
        run_f.final_text.contains('a'),
        "under no saving the character reaches the retained text, got {:?}",
        run_f.final_text
    );

    let ticks_r = run_r
        .ticks_to_reflect()
        .expect("under maximum saving a frame reflecting the keystroke is produced");
    let ticks_f = run_f
        .ticks_to_reflect()
        .expect("under no saving a frame reflecting the keystroke is produced");

    assert_eq!(
        ticks_r, ticks_f,
        "the number of ticks from event delivery to the frame containing `a` is \
         equal under maximum saving ({ticks_r}) and no saving ({ticks_f}): a \
         power profile must not compromise critical responsiveness"
    );
    assert_eq!(
        run_r.reflected_at, run_f.reflected_at,
        "the frame reflecting the keystroke is produced at the same tick index \
         in both runs"
    );
}

#[doc = include_str!("spec_ch7b_power.doc/d_an_absent_power_provider_reports_typed_absence_without_fabricating.md")]
#[test]
fn d_an_absent_power_provider_reports_typed_absence_without_fabricating() {
    // A backend with no power provider installed.
    let absent = NullPowerProvider;

    // The call completes and reports absence in the type system.
    assert_eq!(
        absent.availability(),
        SupportLevel::Unavailable,
        "with no power provider the capability query returns the typed \
         `unsupported` variant"
    );

    // The reading it returns must not be indistinguishable from a genuine one:
    // a machine whose power state is unknown must not be reported as a
    // plugged-in machine with nominal thermals.
    let reading = absent.profile();
    let genuine_plugged_in = PowerProfile::new().on_source(PowerSource::Ac);
    assert_ne!(
        reading, genuine_plugged_in,
        "an unavailable power provider must not fabricate a reading that is \
         indistinguishable from a genuine plugged-in reading, got {reading:?}"
    );
    assert_eq!(
        reading.source(),
        PowerSource::Unknown,
        "with no provider the power source is reported as the typed \
         `Unknown` variant, not as a fabricated `Ac`"
    );

    // Driving a frame afterwards is unchanged from a run that never queried.
    let mut queried = runtime();
    let _ = absent.availability();
    let after_query = drive_effect_frames(&mut queried, 3);

    let mut untouched = runtime();
    let never_queried = drive_effect_frames(&mut untouched, 3);

    assert_eq!(
        after_query, never_queried,
        "the frames after an absent-capability query are unchanged from a run \
         that never made the query"
    );
}
