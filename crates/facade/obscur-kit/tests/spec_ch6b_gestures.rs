#![doc = include_str!("spec_ch6b_gestures.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::identity::NodeId;
use obscur_kit::interaction::gesture::{
    GestureArena, GestureContender, GestureResolution, RecognizerId, RecognizerState,
};

#[doc = include_str!("spec_ch6b_gestures.doc/APP_PAN.md")]
const APP_PAN: u64 = 1;
#[doc = include_str!("spec_ch6b_gestures.doc/SYSTEM_BACK.md")]
const SYSTEM_BACK: u64 = 2;

fn node() -> NodeId {
    NodeId::from_raw(0x6B16)
}

#[doc = include_str!("spec_ch6b_gestures.doc/a_system_gesture_wins_the_arena_by_default.md")]
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

#[doc = include_str!("spec_ch6b_gestures.doc/the_application_pan_is_cancelled_when_the_system_gesture_takes_the_sequence.md")]
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

#[doc = include_str!("spec_ch6b_gestures.doc/an_application_pan_outside_a_reserved_region_wins.md")]
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
