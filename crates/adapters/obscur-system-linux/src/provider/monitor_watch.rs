//! Watching a settings store through a monitoring tool's output.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use obscur_system::appearance::{AppearanceCallback, AppearanceWatch};

#[doc = include_str!("../../docs/api/provider/monitor_watch.doc/MonitorWatch.md")]
#[derive(Debug)]
pub(super) struct MonitorWatch;

impl MonitorWatch {
    #[doc = include_str!("../../docs/api/provider/monitor_watch.doc/spawn.md")]
    pub(super) fn spawn(
        tool: &str,
        args: &[&str],
        marker: &'static str,
        on_change: AppearanceCallback,
    ) -> Option<AppearanceWatch> {
        let mut child = Command::new(tool)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .ok()?;
        let stdout = child.stdout.take()?;

        // Set before the guard is handed out, cleared by it. The reader checks it
        // on every line, so a watch stopped while the tool is mid-sentence does
        // not deliver one last change after the caller let go.
        let live = Arc::new(AtomicBool::new(true));
        let reading = Arc::clone(&live);

        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if !reading.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(line) = line else { break };
                if line.contains(marker) {
                    // Deliberately not "what changed": the callback's contract is
                    // that something did, and the host reads the whole appearance
                    // again. Parsing the value out of a monitor's prose would be a
                    // second parser, for a second format, to save one read.
                    on_change();
                }
            }
        });

        Some(AppearanceWatch::new(move || {
            live.store(false, Ordering::SeqCst);
            stop_child(&mut child);
        }))
    }
}

#[doc = include_str!("../../docs/api/provider/monitor_watch.doc/stop_child.md")]
fn stop_child(child: &mut Child) {
    // Killing closes the pipe, which is what ends the reader thread's `lines()`
    // loop; the thread is left to finish on its own rather than joined, because
    // joining would make dropping the guard wait on a process that may be slow
    // to die, and there is nothing left for it to do.
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;
    use std::time::{Duration, Instant};

    use super::*;

    fn counter() -> (AppearanceCallback, Arc<AtomicUsize>) {
        let seen = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&seen);
        (
            Arc::new(move || {
                counted.fetch_add(1, Ordering::SeqCst);
            }),
            seen,
        )
    }

    #[test]
    fn a_tool_that_is_not_there_yields_no_watch() {
        let (callback, seen) = counter();
        assert!(MonitorWatch::spawn("obscur-no-such-tool", &[], "x", callback).is_none());
        assert_eq!(seen.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a_line_carrying_the_marker_is_a_change_and_another_line_is_not() {
        // `echo` stands in for the monitor: two lines, one of which mentions the
        // namespace. Nothing here needs a desktop, which is the point — the
        // wiring is what breaks, not the tool.
        let (callback, seen) = counter();
        let watch = MonitorWatch::spawn(
            "printf",
            &["unrelated line\\norg.freedesktop.appearance color-scheme\\nanother\\n"],
            "org.freedesktop.appearance",
            callback,
        )
        .expect("printf is on every system that runs these tests");

        let deadline = Instant::now() + Duration::from_secs(5);
        while seen.load(Ordering::SeqCst) == 0 && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            seen.load(Ordering::SeqCst),
            1,
            "one of the three lines mentions the namespace"
        );
        drop(watch);
    }

    #[test]
    fn dropping_the_guard_is_safe_before_anything_arrives() {
        // The ordinary teardown path, and the one an application hits when it
        // exits promptly: the child is killed and the reader ends with it.
        let (callback, _) = counter();
        let watch = MonitorWatch::spawn("sleep", &["30"], "never", callback);
        assert!(watch.is_some(), "sleep exists");
        drop(watch);
    }
}
