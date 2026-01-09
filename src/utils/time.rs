use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current Unix timestamp in seconds.
///
/// # Panics
/// This function will panic if the system clock is set before the Unix epoch (January 1, 1970).
/// This is considered a system misconfiguration and is extremely rare in practice.
#[inline]
pub fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before Unix epoch")
        .as_secs() as i64
}
