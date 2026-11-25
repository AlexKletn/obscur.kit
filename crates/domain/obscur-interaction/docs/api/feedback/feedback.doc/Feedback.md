Cloneable host-configured feedback handle for the UI thread. Each channel is optional. A default handle silently drops requests.

```
use std::{cell::Cell, rc::Rc};
use obscur_interaction::feedback::{Feedback, FeedbackEffect, HapticFeedback};

struct HostHaptics(Rc<Cell<Option<FeedbackEffect>>>);

impl HapticFeedback for HostHaptics {
    fn play(&self, effect: FeedbackEffect) {
        self.0.set(Some(effect));
    }
}

let received = Rc::new(Cell::new(None));
let feedback = Feedback::default().with_haptics(HostHaptics(Rc::clone(&received)));
feedback.sound(FeedbackEffect::Click);
assert_eq!(received.get(), None);
feedback.clone().haptic(FeedbackEffect::Tick);
assert_eq!(received.get(), Some(FeedbackEffect::Tick));
```
