//! Read-only deadline locators. They never create a completed history, lease,
//! mutation window, native Owner or permission to reopen an expired Stop.
use super::*;
use crate::modules::runtime::official_lifecycle024::ActionWindow;
use std::time::{Duration, Instant};
enum Origin {
    Status(Instant),
    Action(ActionWindow),
    StopAction(ActionWindow),
    CompletedOriginal(Instant),
}
pub(super) struct HistoryReadWindow {
    origin: Origin,
    end: Instant,
}
impl HistoryReadWindow {
    pub(super) fn completed_original(end: Instant, original: Instant) -> io::Result<Self> {
        if end != original {
            return Err(invalid());
        }
        let out = Self {
            origin: Origin::CompletedOriginal(original),
            end,
        };
        out.check()?;
        Ok(out)
    }
    pub(super) fn requires_original_end(&self) -> bool {
        matches!(self.origin, Origin::CompletedOriginal(_))
    }
    pub(super) fn status(when: Instant, end: Instant) -> io::Result<Self> {
        let out = Self {
            origin: Origin::Status(when),
            end,
        };
        out.check()?;
        Ok(out)
    }
    pub(super) fn action(window: &ActionWindow) -> io::Result<Self> {
        window
            .remaining()
            .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))?;
        let out = Self {
            origin: Origin::Action(*window),
            end: window.deadline(),
        };
        out.check()?;
        Ok(out)
    }
    pub(super) fn stop_action(window: &ActionWindow) -> io::Result<Self> {
        let out = Self {
            origin: Origin::StopAction(*window),
            end: window.stop_deadline(),
        };
        out.check()?;
        Ok(out)
    }
    pub(super) fn check(&self) -> io::Result<()> {
        if Instant::now() >= self.end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        match &self.origin {
            Origin::CompletedOriginal(end) => {
                if *end != self.end {
                    return Err(invalid());
                }
            }
            Origin::Status(when) => {
                if *when > Instant::now()
                    || *when >= self.end
                    || self.end.duration_since(*when) > Duration::from_secs(5)
                {
                    return Err(invalid());
                }
            }
            Origin::Action(window) => {
                if self.end != window.deadline() {
                    return Err(invalid());
                }
                window
                    .remaining()
                    .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))?;
            }
            Origin::StopAction(window) => {
                if self.end != window.stop_deadline() || self.end > window.deadline() {
                    return Err(invalid());
                }
                window
                    .remaining()
                    .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))?;
            }
        }
        Ok(())
    }
    pub(super) fn end(&self) -> io::Result<Instant> {
        self.check()?;
        Ok(self.end)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::process::LifecycleAction;
    #[test]
    fn wl2_r6_ready_action_read_keeps_actual_window_without_relabeling_it_status() {
        let start = Instant::now();
        for kind in [
            LifecycleAction::Start,
            LifecycleAction::Stop,
            LifecycleAction::Restart,
        ] {
            let window = ActionWindow::begin(kind, start, start + Duration::from_secs(30)).unwrap();
            let read = HistoryReadWindow::action(&window).unwrap();
            let stop = HistoryReadWindow::stop_action(&window).unwrap();
            assert_eq!(read.end().unwrap(), window.deadline());
            assert_eq!(stop.end().unwrap(), window.stop_deadline());
            assert!(
                HistoryReadWindow::status(
                    window.deadline() - Duration::from_secs(5),
                    window.deadline()
                )
                .is_err(),
                "future deterministic Status start must NOT be used for a longer original action"
            );
        }
        println!("r6 Ready observation action windows retain actual Start30/Stop10/Restart outer30+sub10; Status future/over5 checks unchanged. No Owner or typed Ready factory from these locators");
    }
    #[test]
    fn wl2_r6_predecessor_stop_read_uses_original_subwindow_even_while_restart_outer_is_live() {
        let start = Instant::now() - Duration::from_secs(9);
        let window = ActionWindow::begin(
            LifecycleAction::Restart,
            start,
            start + Duration::from_secs(30),
        )
        .unwrap();
        let read = HistoryReadWindow::stop_action(&window).unwrap();
        assert_eq!(read.end().unwrap(), window.stop_deadline());
        assert!(read.end().unwrap() < window.deadline());
        while Instant::now() < window.stop_deadline() {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(
            window.remaining().is_ok(),
            "original Restart outer is still live"
        );
        assert_eq!(read.end().err().unwrap().kind(), io::ErrorKind::TimedOut);
        assert_eq!(
            HistoryReadWindow::stop_action(&window)
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::TimedOut
        );
        assert_eq!(window.stop_deadline(), start + Duration::from_secs(10));
        println!("r6 native predecessor readonly Stop keeps exact original Restart sub10; real elapsed subwindow refuses even with live outer30, no retry/budget reset or native authority");
    }
    #[test]
    fn wl2_r6_read_window_original_completion_requires_exact_original_cutoff() {
        let original = Instant::now() + Duration::from_secs(2);
        let read = HistoryReadWindow::completed_original(original, original).unwrap();
        assert!(read.requires_original_end());
        assert_eq!(read.end().unwrap(), original);
        assert!(HistoryReadWindow::completed_original(
            original + Duration::from_nanos(1),
            original
        )
        .is_err());
        assert!(HistoryReadWindow::completed_original(
            original - Duration::from_nanos(1),
            original
        )
        .is_err());
        let old = Instant::now() - Duration::from_nanos(1);
        assert!(HistoryReadWindow::completed_original(old, old).is_err());
        println!("r6 original completed-Stop read locator requires exact old end, not shortened/extended/expired. Journal separately binds end to original native transaction; locator alone grants no completion");
    }
    #[test]
    fn wl2_r6_read_window_status_keeps_same_sample_and_refuses_unknown_or_renewed_bound() {
        let when = Instant::now();
        let end = when + Duration::from_secs(5);
        let read = HistoryReadWindow::status(when, end).unwrap();
        assert_eq!(read.end().unwrap(), end);
        assert!(HistoryReadWindow::status(when, end + Duration::from_nanos(1)).is_err());
        assert!(HistoryReadWindow::status(
            when + Duration::from_secs(60),
            end + Duration::from_secs(60)
        )
        .is_err());
        assert!(HistoryReadWindow::status(when, when).is_err());
        assert!(HistoryReadWindow::status(
            when,
            end - Duration::from_secs(5) - Duration::from_nanos(1)
        )
        .is_err());
        assert_eq!(read.end().unwrap(), end, "no later check constructs now+5");
        println!("r6 readonly Status window preserves exact original when/end, <=5s, no future/expired/renewed sample; locator grants no native history or mutation");
    }
    #[test]
    fn wl2_r6_read_window_actions_keep_borrowed_original_outer_and_stop_deadlines() {
        for kind in [
            LifecycleAction::Start,
            LifecycleAction::Stop,
            LifecycleAction::Restart,
        ] {
            let start = Instant::now();
            let parent = start + Duration::from_secs(2);
            let window = ActionWindow::begin(kind, start, parent).unwrap();
            let end = window.deadline();
            let stop = window.stop_deadline();
            let read = HistoryReadWindow::action(&window).unwrap();
            assert_eq!(read.end().unwrap(), end);
            assert_eq!(end, parent);
            assert_eq!(stop, parent);
            assert_eq!(window.deadline(), end);
            assert_eq!(window.stop_deadline(), stop);
            assert!(ActionWindow::begin(kind, start - Duration::from_secs(60), parent).is_err());
        }
        let start = Instant::now();
        let parent = start + Duration::from_secs(60);
        let restart = ActionWindow::begin(LifecycleAction::Restart, start, parent).unwrap();
        let read = HistoryReadWindow::action(&restart).unwrap();
        assert_eq!(read.end().unwrap(), restart.deadline());
        assert_eq!(restart.deadline(), start + Duration::from_secs(30));
        assert_eq!(restart.stop_deadline(), start + Duration::from_secs(10));
        println!("r6 readonly action window consumes original ActionWindow deadline exactly; caller parent shrinks Start/Stop/Restart, original Restart outer30/Stop10 unchanged; no new budget/adoption/native permit");
    }
    #[test]
    fn wl2_r6_read_window_expired_original_action_stays_timeout_without_new_deadline() {
        let start = Instant::now();
        let window = ActionWindow::begin(
            LifecycleAction::Start,
            start,
            start + Duration::from_secs(1),
        )
        .unwrap();
        let read = HistoryReadWindow::action(&window).unwrap();
        let end = read.end().unwrap();
        while Instant::now() < end {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(read.end().err().unwrap().kind(), io::ErrorKind::TimedOut);
        assert_eq!(
            HistoryReadWindow::action(&window).err().unwrap().kind(),
            io::ErrorKind::TimedOut
        );
        assert_eq!(window.deadline(), end);
        assert_eq!(window.stop_deadline(), end);
        println!("r6 real elapsed original ActionWindow retains TimedOut both on original read and attempted later construction; no replacement window/deadline or native permission");
    }
}
