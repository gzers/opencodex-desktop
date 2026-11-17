//! Read-only deadline locators. They never create a completed history, lease,
//! mutation window, native Owner or permission to reopen an expired Stop.
use super::*;
use crate::modules::runtime::official_lifecycle024::ActionWindow;
use std::time::{Duration, Instant};
enum Origin {
    Status(Instant),
    Action(ActionWindow),
}
pub(super) struct HistoryReadWindow {
    origin: Origin,
    end: Instant,
}
impl HistoryReadWindow {
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
    pub(super) fn check(&self) -> io::Result<()> {
        if Instant::now() >= self.end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        match &self.origin {
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
