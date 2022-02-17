use crate::error::Error;
use std::time::Duration;

pub fn default_should_retry(error: &Error) -> bool {
    match error {
        Error::Utf8(_)
        | Error::Parser(_)
        | Error::InvalidStatusCode(_)
        | Error::InvalidContentType(_) => false,
        Error::Transport(_) | Error::StreamEnded => true,
    }
}

pub trait RetryPolicy {
    fn retry(&self, error: &Error, last_retry: Option<(usize, Duration)>) -> Option<Duration>;
}

#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    pub start: Duration,
    pub factor: f64,
    pub max_duration: Duration,
    pub max_retries: Option<usize>,
}

impl ExponentialBackoff {
    pub const fn new(
        start: Duration,
        factor: f64,
        max_duration: Duration,
        max_retries: Option<usize>,
    ) -> Self {
        Self {
            start,
            factor,
            max_duration,
            max_retries,
        }
    }
}

impl RetryPolicy for ExponentialBackoff {
    fn retry(&self, error: &Error, last_retry: Option<(usize, Duration)>) -> Option<Duration> {
        if !default_should_retry(error) {
            return None;
        }
        if let Some((retry_num, last_duration)) = last_retry {
            if self.max_retries.is_none() || retry_num < self.max_retries.unwrap() {
                Some(last_duration.mul_f64(self.factor).min(self.max_duration))
            } else {
                None
            }
        } else {
            Some(self.start)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub delay: Duration,
    pub max_retries: Option<usize>,
}

impl Constant {
    pub const fn new(delay: Duration, max_retries: Option<usize>) -> Self {
        Self { delay, max_retries }
    }
}

impl RetryPolicy for Constant {
    fn retry(&self, error: &Error, last_retry: Option<(usize, Duration)>) -> Option<Duration> {
        if !default_should_retry(error) {
            return None;
        }
        if let Some((retry_num, _)) = last_retry {
            if self.max_retries.is_none() || retry_num < self.max_retries.unwrap() {
                Some(self.delay)
            } else {
                None
            }
        } else {
            Some(self.delay)
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Never;

impl RetryPolicy for Never {
    fn retry(&self, _error: &Error, _last_retry: Option<(usize, Duration)>) -> Option<Duration> {
        None
    }
}

pub const DEFAULT_RETRY: ExponentialBackoff =
    ExponentialBackoff::new(Duration::from_millis(300), 2., Duration::from_secs(5), None);
