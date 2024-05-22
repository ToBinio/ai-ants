use std::fmt::{Debug, Formatter};
use std::time::Duration;

#[derive(Default)]
pub struct AvgDuration {
    duration: Duration,
}

impl AvgDuration {
    pub fn add(&mut self, duration: &Duration) {
        self.duration = avg_duration(&self.duration, duration);
    }
}

impl Debug for AvgDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.duration))
    }
}

pub fn avg_duration(current: &Duration, new: &Duration) -> Duration {
    Duration::from_secs_f64((current.as_secs_f64() * 59. + new.as_secs_f64()) / 60.)
}
