//! [`BackgroundWork`]: the §3.6 band-5 work queue's accounting.

use super::background_serviced::BackgroundServiced;

use obscur_task::CancellationToken;
use std::collections::VecDeque;
use std::time::Duration;

type BackgroundJob = (
    Option<CancellationToken>,
    Option<Duration>,
    Box<dyn FnOnce()>,
);

#[doc = include_str!("../../docs/api/runtime/background_work.doc/BackgroundWork.md")]
#[derive(Default)]
pub(crate) struct BackgroundWork {
    /// Units enqueued and not yet serviced.
    pending: usize,
    /// Units serviced over this runtime's whole life — never reset, so it is the
    /// observable proof that background work made progress under a stream of
    /// higher-priority frames.
    serviced: u64,
    /// Runnable units in the order in which they were enqueued.
    jobs: VecDeque<BackgroundJob>,
}

impl BackgroundWork {
    /// An empty queue that has serviced nothing.
    pub(crate) const fn new() -> Self {
        Self {
            pending: 0,
            serviced: 0,
            jobs: VecDeque::new(),
        }
    }

    #[doc = include_str!("../../docs/api/runtime/background_work.doc/request.md")]
    pub(crate) const fn request(&mut self) -> bool {
        self.pending = self.pending.saturating_add(1);
        self.pending == 1
    }

    /// Adds a runnable unit and reports whether it started a pending batch.
    pub(crate) fn enqueue(
        &mut self,
        cancellation: Option<CancellationToken>,
        deadline: Option<Duration>,
        job: impl FnOnce() + 'static,
    ) -> bool {
        self.jobs.push_back((cancellation, deadline, Box::new(job)));
        self.request()
    }

    /// Takes the oldest queued callback with its cancellation token and deadline,
    /// when supplied.
    pub(crate) fn take_job(&mut self) -> Option<BackgroundJob> {
        self.jobs.pop_front()
    }

    /// Removes a taken job from the pending count without counting it as serviced.
    pub(crate) fn discard_job(&mut self) {
        self.pending -= 1;
    }

    /// Units still waiting to be serviced.
    pub(crate) const fn pending(&self) -> usize {
        self.pending
    }

    /// Units serviced so far.
    pub(crate) const fn serviced(&self) -> u64 {
        self.serviced
    }

    #[doc = include_str!("../../docs/api/runtime/background_work.doc/service.md")]
    pub(crate) const fn service(&mut self) -> Option<BackgroundServiced> {
        if self.pending == 0 {
            return None;
        }
        self.pending -= 1;
        self.serviced = self.serviced.saturating_add(1);
        Some(if self.pending == 0 {
            BackgroundServiced::Drained
        } else {
            BackgroundServiced::More
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{BackgroundServiced, BackgroundWork};

    #[test]
    fn an_empty_queue_services_nothing() {
        let mut work = BackgroundWork::new();
        assert_eq!(work.pending(), 0);
        assert_eq!(work.serviced(), 0);
        assert!(
            work.service().is_none(),
            "no unit ran, so the scheduler must not be told one did"
        );
        // And nothing was invented on the way out.
        assert_eq!(work.serviced(), 0);
    }

    #[test]
    fn only_the_first_request_and_the_last_service_report_an_edge() {
        // The signal is edge-triggered in both directions. Re-announcing "pending"
        // on the second unit would restart the fairness count the first is waiting
        // on; missing "drained" on the last one leaves the scheduler reporting
        // starvation for work that no longer exists.
        let mut work = BackgroundWork::new();
        assert!(work.request(), "0 → 1 is the announcement");
        assert!(
            !work.request(),
            "a second unit joins a queue already announced"
        );
        assert!(!work.request());
        assert_eq!(work.pending(), 3);

        assert_eq!(work.service(), Some(BackgroundServiced::More));
        assert_eq!(work.service(), Some(BackgroundServiced::More));
        assert_eq!(work.service(), Some(BackgroundServiced::Drained));
        assert_eq!(work.pending(), 0);
        // Drained means drained: the next request announces again.
        assert!(work.request());
    }

    #[test]
    fn a_serviced_unit_leaves_the_queue_exactly_as_it_enters_the_total() {
        // The pairing itself: after any number of requests and services, the two
        // tallies must still add up to what was enqueued.
        let mut work = BackgroundWork::new();
        let mut enqueued = 0_u64;
        for round in 1..=5_u64 {
            for _ in 0..round {
                let _ = work.request();
                enqueued += 1;
            }
            let _ = work.service();
            assert_eq!(
                work.pending() as u64 + work.serviced(),
                enqueued,
                "round {round}: a unit is either waiting or counted, never both or neither"
            );
        }
    }

    #[test]
    fn the_tallies_saturate_rather_than_wrap() {
        // Unreachable through the runtime — a u64 of serviced units is not
        // countable in a session — but a wrap here would turn the progress proof
        // into a claim that background work went backwards.
        let mut work = BackgroundWork {
            pending: 1,
            serviced: u64::MAX,
            ..BackgroundWork::new()
        };
        assert_eq!(work.service(), Some(BackgroundServiced::Drained));
        assert_eq!(work.serviced(), u64::MAX);

        let mut full = BackgroundWork {
            pending: usize::MAX,
            serviced: 0,
            ..BackgroundWork::new()
        };
        assert!(!full.request());
        assert_eq!(full.pending(), usize::MAX);
    }
}
