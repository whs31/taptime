#[derive(Default)]
pub struct NotificationSchedule {
  date: Option<chrono::NaiveDate>,
  previous: Option<i64>,
  fired: [bool; 4],
}

impl NotificationSchedule {
  pub fn update(
    &mut self,
    date: chrono::NaiveDate,
    remaining_seconds: i64,
    active: bool,
    stale: bool,
  ) -> Vec<i64> {
    if stale {
      return Vec::new();
    }
    if self.date != Some(date) {
      self.date = Some(date);
      self.previous = None;
      self.fired = [false; 4];
    }
    if !active {
      return Vec::new();
    }
    let mut due = Vec::new();
    for (i, threshold) in [14400, 7200, 3600, 1800].into_iter().enumerate() {
      if !self.fired[i]
        && self.previous.is_some_and(|p| p > threshold)
        && remaining_seconds <= threshold
      {
        self.fired[i] = true;
        due.push(threshold);
      }
    }
    self.previous = Some(remaining_seconds);
    due
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn crossing_is_delivered_once_and_stale_is_ignored() {
    let date = chrono::NaiveDate::from_ymd_opt(2026, 11, 13).unwrap();
    let mut schedule = NotificationSchedule::default();
    assert!(schedule.update(date, 15000, true, false).is_empty());
    assert!(schedule.update(date, 14000, true, true).is_empty());
    assert_eq!(schedule.update(date, 14000, true, false), vec![14400]);
    assert!(schedule.update(date, 14000, true, false).is_empty());
    assert!(
      schedule
        .update(date.succ_opt().unwrap(), 14000, true, false)
        .is_empty()
    );
  }
}
