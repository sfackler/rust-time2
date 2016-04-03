use std::time::Duration;

const NANOS_PER_SEC: u64 = 1_000_000_000;
const MILLIS_PER_SEC: u64 = 1_000;
const NANOS_PER_MILLI: u64 = NANOS_PER_SEC / MILLIS_PER_SEC;

/// Extensions to the `Duration` type.
pub trait DurationExt {
    /// Returns the number of whole milliseconds contained in this `Duration`.
    ///
    /// Returns `None` if the value exceeds the capacity of a `u64.
    fn as_millis(&self) -> Option<u64>;
}

impl DurationExt for Duration {
    fn as_millis(&self) -> Option<u64> {
        self.as_secs()
            .checked_mul(1000)
            .and_then(|m| m.checked_add(self.subsec_nanos() as u64 / NANOS_PER_MILLI))
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;

    #[test]
    fn as_millis() {
        assert_eq!(Some(1100), Duration::new(1, 100_000_999).as_millis());
        assert_eq!(None, Duration::from_secs(1 << 60).as_millis());
    }
}
