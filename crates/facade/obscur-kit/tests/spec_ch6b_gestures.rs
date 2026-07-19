//! §6.16 Mobile gestures — the arena's default priority direction.
//!
//! Specification-derived. The load-bearing sentence is "System gestures have
//! priority by default": system priority is the default and application priority
//! is the opt-in, not the other way round.
//!
//! NOTE ON THE SETUP. §6.16 also names "edge/reserved gesture regions" and
//! "exclusion regions where allowed". This crate exposes no way to declare a
//! reserved system-gesture region, no way to mark a contender as a *system*
//! gesture, and no exclusion-region registration API, so the case's scenario 2
//! (the app opts out of the reserved strip and then wins) cannot be written.
//! What is written here is scenario 1 — the default direction — expressed
//! through the only arena surface that exists, and scenario 3 as its control.

#![forbid(unsafe_code)]

use obscur_kit::identity::NodeId;
use obscur_kit::interaction::gesture::{
    GestureArena, GestureContender, GestureResolution, RecognizerId, RecognizerState,
};

/// The application's full-bleed horizontal pan, attached first (it covers the
/// whole viewport, reserved strip included).
const APP_PAN: u64 = 1;
/// The host's reserved left-edge system back gesture, entered second.
const SYSTEM_BACK: u64 = 2;

fn node() -> NodeId {
    NodeId::from_raw(0x6B16)
}

/// §6.16 "- edge/reserved gesture regions;" / "- gesture arena;" /
/// "System gestures have priority by default."
///
/// A horizontal drag starting at x=8, inside a 20px reserved system-back strip:
/// the system gesture wins the arena and the application's pan recognizer does
/// not become the winner. Neither recognizer is given an explicit priority —
/// system priority is the *default*, not something the caller opts into.
#[test]
fn a_system_gesture_wins_the_arena_by_default() {
    let app_pan = RecognizerId::from_raw(APP_PAN);
    let system_back = RecognizerId::from_raw(SYSTEM_BACK);

    let mut arena = GestureArena::new();
    arena.add(GestureContender::new(app_pan, node()));
    arena.add(GestureContender::new(system_back, node()));
    arena.mark(app_pan, RecognizerState::Recognized);
    arena.mark(system_back, RecognizerState::Recognized);

    assert_eq!(
        arena.sweep(),
        GestureResolution::Winner(system_back),
        "the reserved system back gesture must win by default against a \
         full-bleed application pan that neither declared nor was granted any \
         priority"
    );
}

/// §6.16 "- gesture arena;" / "System gestures have priority by default."
///
/// The application's pan recognizer must receive no pan updates once the system
/// gesture has taken the sequence: a cancellation is acceptable, a silent pan is
/// not.
#[test]
fn the_application_pan_is_cancelled_when_the_system_gesture_takes_the_sequence() {
    let app_pan = RecognizerId::from_raw(APP_PAN);
    let system_back = RecognizerId::from_raw(SYSTEM_BACK);

    let mut arena = GestureArena::new();
    arena.add(GestureContender::new(app_pan, node()));
    arena.add(GestureContender::new(system_back, node()));
    arena.mark(app_pan, RecognizerState::Recognized);
    arena.mark(system_back, RecognizerState::Recognized);
    let _ = arena.sweep();

    let app_state = arena
        .contenders()
        .iter()
        .find(|(c, _)| c.recognizer() == app_pan)
        .map(|(_, state)| *state)
        .expect("the application pan is still a tracked contender");

    assert_eq!(
        app_state,
        RecognizerState::Cancelled,
        "the application pan must be cancelled, not left recognized, once the \
         system back gesture owns the sequence"
    );
}

/// §6.16 "- gesture arena;" (control, scenario 3).
///
/// A horizontal drag starting at x=200, outside any reserved region: no system
/// gesture contends, and the application's pan wins.
#[test]
fn an_application_pan_outside_a_reserved_region_wins() {
    let app_pan = RecognizerId::from_raw(APP_PAN);

    let mut arena = GestureArena::new();
    arena.add(GestureContender::new(app_pan, node()));
    arena.mark(app_pan, RecognizerState::Recognized);

    assert_eq!(
        arena.sweep(),
        GestureResolution::Winner(app_pan),
        "with no system gesture contending, the application pan wins regardless"
    );
}
