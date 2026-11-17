//! Conservative native QPC projection retained ONCE by the original action.
//! QPC fields are diagnostic record DATA; they cannot extend Instant authority.
use std::{
    io,
    time::{Duration, Instant},
};
pub(super) struct RecoveryClock {
    start: u64,
    cutoff: u64,
    frequency: u64,
    end: Instant,
}
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "original recovery clock differs",
    )
}
impl RecoveryClock {
    pub(super) fn check_history(&self, original_end: Instant, read_end: Instant) -> io::Result<()> {
        if Instant::now() >= read_end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        let (counter, frequency) = crate::infrastructure::windows_owner_host024::qpc()?;
        if original_end != self.end
            || frequency != self.frequency
            || counter < self.start
            || self.cutoff <= self.start
            || frequency
                .checked_mul(10)
                .is_none_or(|max| self.cutoff - self.start > max)
        {
            return Err(invalid());
        }
        if Instant::now() >= read_end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(())
    }
    pub(super) fn capture(end: Instant) -> io::Result<Self> {
        let (counter, frequency) = crate::infrastructure::windows_owner_host024::qpc()?;
        // Sampling the remaining duration AFTER the QPC read rounds the record
        // deadline down, never beyond the inherited exact Instant cutoff.
        Self::sample(counter, frequency, Instant::now(), end)
    }
    fn sample(counter: u64, frequency: u64, after: Instant, end: Instant) -> io::Result<Self> {
        if frequency == 0 || after >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        let remaining = end.duration_since(after);
        if remaining > Duration::from_secs(10) {
            return Err(invalid());
        }
        let ticks = remaining
            .as_nanos()
            .checked_mul(frequency as u128)
            .ok_or_else(invalid)?
            / 1_000_000_000;
        let ticks = u64::try_from(ticks).map_err(|_| invalid())?;
        let cutoff = counter
            .checked_add(ticks)
            .filter(|v| *v > counter)
            .ok_or_else(invalid)?;
        if ticks > frequency.checked_mul(10).ok_or_else(invalid)? {
            return Err(invalid());
        }
        Ok(Self {
            start: counter,
            cutoff,
            frequency,
            end,
        })
    }
    fn check_sample(
        &self,
        end: Instant,
        now: Instant,
        counter: u64,
        frequency: u64,
    ) -> io::Result<()> {
        if end != self.end || frequency != self.frequency || counter < self.start {
            return Err(invalid());
        }
        if now >= end || counter >= self.cutoff {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(())
    }
    pub(super) fn check(&self, end: Instant) -> io::Result<()> {
        let (counter, frequency) = crate::infrastructure::windows_owner_host024::qpc()?;
        self.check_sample(end, Instant::now(), counter, frequency)
    }
    pub(super) fn data(&self) -> [String; 3] {
        [
            self.start.to_string(),
            self.cutoff.to_string(),
            self.frequency.to_string(),
        ]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wl2_r7_full_history_native_qpc_metadata_does_not_reopen_original_clock() {
        let end = Instant::now() + Duration::from_millis(60);
        let mut clock = RecoveryClock::capture(end).unwrap();
        let original = clock.data();
        while Instant::now() < end {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            clock.check(end).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        let read_end = Instant::now() + Duration::from_secs(5);
        clock.check_history(end, read_end).unwrap();
        assert_eq!(clock.data(), original);
        assert_eq!(clock.end, end);
        assert!(clock
            .check_history(end + Duration::from_nanos(1), read_end)
            .is_err());
        assert!(clock.check_history(end, Instant::now()).is_err());
        clock.frequency += 1;
        assert!(clock.check_history(end, read_end).is_err());
        assert_eq!(clock.end, end);
    }
    #[test]
    fn wl2_r7_attachment_origin_clock_native_fixed_projection_never_resamples_or_renews() {
        let end = Instant::now() + Duration::from_secs(2);
        let clock = RecoveryClock::capture(end).unwrap();
        let first = clock.data();
        clock.check(end).unwrap();
        clock.check(end).unwrap();
        assert_eq!(clock.data(), first);
        let frequency = clock.frequency;
        let span = clock.cutoff - clock.start;
        assert!(span > 0 && span <= frequency * 2);
        assert!(clock.check(end + Duration::from_nanos(1)).is_err());
        assert_eq!(clock.end, end);
    }
    #[test]
    fn wl2_r7_attachment_origin_clock_projection_rejects_unknown_expiry_frequency_overflow_and_rounding(
    ) {
        let now = Instant::now();
        let end = now + Duration::from_secs(1);
        let clock = RecoveryClock::sample(100, 1000, now, end).unwrap();
        assert_eq!(clock.data(), ["100", "1100", "1000"].map(String::from));
        assert!(clock.check_sample(end, now, 99, 1000).is_err());
        assert!(clock.check_sample(end, now, 100, 1001).is_err());
        assert!(clock.check_sample(end, end, 100, 1000).is_err());
        assert!(clock.check_sample(end, now, 1100, 1000).is_err());
        assert!(RecoveryClock::sample(u64::MAX, 1000, now, end).is_err());
        assert!(RecoveryClock::sample(0, 0, now, end).is_err());
        assert!(RecoveryClock::sample(0, 1, now, now + Duration::from_nanos(1)).is_err());
        assert!(RecoveryClock::sample(0, 1000, now, now + Duration::from_secs(11)).is_err());
        assert!(RecoveryClock::sample(0, 1000, now, now).is_err());
    }
}
