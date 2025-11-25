use std::{fs::File, io::BufReader, path::PathBuf};

use chrono::{DateTime, Duration, Utc};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{from_reader, to_writer};

use crate::{Error, Interval, Result};

/// Splits a time range into intervals suitable for Binance's API (max 1000 candles per request).
///
/// # Arguments
/// * `start` - Start date of the range.
/// * `end` - End date of the range.
/// * `interval` - The time interval (m1, h1, d1).
///
/// # Returns
/// A vector of tuples `(start, end)` representing the split intervals.
pub(crate) fn split_intervals(start: DateTime<Utc>, end: DateTime<Utc>, interval: &Interval) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    let mut intervals = Vec::new();
    let mut current_start = start;

    // maximum duration for 1000 candles, based on the interval.
    // also return the interval duration to increment `current_start`.
    let (max_duration, plus_duration) = match interval {
        Interval::S1 => (Duration::minutes(1000), Duration::seconds(1)),
        Interval::M1 => (Duration::minutes(1000), Duration::minutes(1)),
        Interval::M3 => (Duration::minutes(1000), Duration::minutes(3)),
        Interval::M5 => (Duration::minutes(1000), Duration::minutes(5)),
        Interval::M15 => (Duration::minutes(1000), Duration::minutes(15)),
        Interval::M30 => (Duration::minutes(1000), Duration::minutes(30)),
        Interval::H1 => (Duration::hours(1000), Duration::hours(1)),
        Interval::H2 => (Duration::hours(1000), Duration::hours(2)),
        Interval::H4 => (Duration::hours(1000), Duration::hours(4)),
        Interval::H6 => (Duration::hours(1000), Duration::hours(6)),
        Interval::H8 => (Duration::hours(1000), Duration::hours(8)),
        Interval::H12 => (Duration::hours(1000), Duration::hours(12)),
        Interval::D1 => (Duration::days(1000), Duration::days(1)),
        Interval::D3 => (Duration::hours(1000), Duration::days(3)),
        Interval::W1 => (Duration::hours(1000), Duration::weeks(1)),
        Interval::MM1 => (Duration::hours(1000), Duration::weeks(4)),
    };

    while current_start < end {
        let current_end = std::cmp::min(current_start + max_duration, end);
        intervals.push((current_start, current_end));
        current_start = current_end + plus_duration; // avoid overlapping
    }

    intervals
}

/// Reads candlestick data from a file containing serialized Kline data.
pub(crate) fn read_data_from_file<T>(path: &PathBuf) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    from_reader(reader).map_err(Error::from)
}

/// Writes candlestick data to a file.
pub(crate) fn write_to_file<T>(path: &PathBuf, klines: &[T]) -> Result<()>
where
    T: Serialize,
{
    let file = File::create(path)?;
    to_writer(file, &klines).map_err(Error::from)
}
