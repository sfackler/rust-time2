use std::time::{UNIX_EPOCH, SystemTime};
use std::i64;

use DurationExt;

/// Extensions to the `SystemTime` type.
///
/// Requires the `beta` feature.
pub trait SystemTimeExt {
    /// Returns the number of whole milliseconds between this time and the Unix
    /// Epoch.
    ///
    /// Returns `None` if the value exceeds the capacity of an `i64`.
    fn as_unix_millis(&self) -> Option<i64>;
}

impl SystemTimeExt for SystemTime {
    fn as_unix_millis(&self) -> Option<i64> {
        match self.duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_millis().and_then(to_i64),
            Err(e) => e.duration().as_millis().and_then(to_i64).map(|n| -n),
        }
    }
}

fn to_i64(n: u64) -> Option<i64> {
    if n <= i64::max_value() as u64 {
        Some(n as i64)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use std::time::{UNIX_EPOCH, Duration};

    use super::*;

    #[test]
    fn as_unix_millis() {
        assert_eq!(Some(100),
                   (UNIX_EPOCH + Duration::from_millis(100)).as_unix_millis());
        assert_eq!(Some(-100),
                   (UNIX_EPOCH - Duration::from_millis(100)).as_unix_millis());
        assert_eq!(None,
                   (UNIX_EPOCH - Duration::from_secs(1 << 60)).as_unix_millis());
    }
}
