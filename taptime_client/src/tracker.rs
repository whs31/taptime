use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct WorkTracker {
  worked: Duration,
  synced_at: Instant,
  active: bool,
}

impl WorkTracker {
  pub fn new(worked: Duration, active: bool) -> Self {
    Self {
      worked,
      synced_at: Instant::now(),
      active,
    }
  }

  pub fn worked_at(&self, now: Instant) -> Duration {
    if self.active {
      self
        .worked
        .saturating_add(now.saturating_duration_since(self.synced_at))
    } else {
      self.worked
    }
  }

  pub fn worked(&self) -> Duration {
    self.worked_at(Instant::now())
  }

  pub fn is_active(&self) -> bool {
    self.active
  }

  pub fn sync(&mut self, worked: Duration, active: bool, now: Instant) {
    self.worked = worked;
    self.active = active;
    self.synced_at = now;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const ONE_MIN: Duration = Duration::from_secs(60);
  const TEN_MIN: Duration = Duration::from_secs(600);

  #[test]
  fn overflow_saturates() {
    let tracker = WorkTracker::new(Duration::MAX, true);
    assert_eq!(
      tracker.worked_at(tracker.synced_at + TEN_MIN),
      Duration::MAX
    );
  }

  #[test]
  fn active_work() {
    let tracker = WorkTracker::new(ONE_MIN, true);
    assert_eq!(
      tracker.worked_at(tracker.synced_at + TEN_MIN),
      ONE_MIN + TEN_MIN
    );
  }

  #[test]
  fn paused_work() {
    let tracker = WorkTracker::new(ONE_MIN, false);
    assert_eq!(tracker.worked_at(tracker.synced_at + TEN_MIN), ONE_MIN);
  }

  #[test]
  fn initial_work() {
    let tracker = WorkTracker::new(ONE_MIN, true);
    assert_eq!(tracker.worked_at(tracker.synced_at), ONE_MIN);
  }

  #[test]
  fn sync_stops_work() {
    let mut tracker = WorkTracker::new(ONE_MIN, true);
    let sync_time = tracker.synced_at + TEN_MIN;
    tracker.sync(ONE_MIN + TEN_MIN, false, sync_time);
    assert_eq!(tracker.worked_at(sync_time + TEN_MIN), ONE_MIN + TEN_MIN);
  }

  #[test]
  fn sync_resumes_work() {
    let mut tracker = WorkTracker::new(ONE_MIN, false);
    let sync_time = tracker.synced_at + TEN_MIN;
    tracker.sync(ONE_MIN, true, sync_time);
    assert_eq!(tracker.worked_at(sync_time + TEN_MIN), ONE_MIN + TEN_MIN);
  }

  #[test]
  fn earlier_instant_is_safe() {
    let tracker = WorkTracker::new(ONE_MIN, true);
    assert_eq!(tracker.worked_at(tracker.synced_at - TEN_MIN), ONE_MIN);
  }
}
