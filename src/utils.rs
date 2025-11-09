use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde_json::{from_reader, to_writer};

use crate::{Data, Interval, Kline};

/// Splits a time range into intervals suitable for Binance's API (max 1000 candles per request).
///
/// # Arguments
/// * `start` - Start date of the range.
/// * `end` - End date of the range.
/// * `interval` - The time interval (m1, h1, d1).
///
/// # Returns
/// A vector of tuples `(start, end)` representing the split intervals.
pub(crate) fn split_intervals(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    interval: &Interval,
) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    let mut intervals = Vec::new();
    let mut current_start = start;

    // maximum duration for 1000 candles, based on the interval.
    // also return the interval duration to increment `current_start`.
    let (max_duration, plus_duration) = match interval {
        Interval::M1 => (Duration::minutes(1000), Duration::minutes(1)),
        Interval::H1 => (Duration::hours(1000), Duration::hours(1)),
        Interval::D1 => (Duration::days(1000), Duration::days(1)),
    };

    while current_start < end {
        let current_end = std::cmp::min(current_start + max_duration, end);
        intervals.push((current_start, current_end));
        current_start = current_end + plus_duration; // avoid overlapping
    }

    intervals
}

/// Reads the last candlestick's close time from a file containing serialized Kline data.
pub(crate) fn read_data_from_file(path: PathBuf) -> Result<Vec<Kline>> {
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let data = from_reader::<_, Vec<Data>>(reader)?;
    let klines = data.into_iter().map(Into::into).collect();

    Ok(klines)
}

/// This function is useful for resuming operations from the last saved point in case of errors.
pub(crate) fn get_last_close_time_from_file(path: PathBuf) -> Result<DateTime<Utc>> {
    let klines = read_data_from_file(path)?;
    klines
        .last()
        .map(|kline| kline.close_time())
        .ok_or(anyhow::Error::msg("file is empty"))
}

pub(crate) fn write_data_to_file(path: PathBuf, klines: &[Kline]) -> Result<()> {
    let file = File::create(&path)?;
    to_writer(file, klines)?;

    Ok(())
}

/// Asynchronously fetches a URL with configurable retry logic.
///
/// This function attempts to send a GET request to the specified URL.
/// If the request fails, it will retry up to `retry` times, pausing for `pause` seconds between each attempt.
///
/// # Arguments
///
/// * `url` - A string slice that holds the URL to fetch.
/// * `retry` - The maximum number of retry attempts if the request fails.
/// * `pause` - The delay in seconds between retry attempts.
///
/// # Returns
///
/// * `Ok(reqwest::Response)` - If the request succeeds within the allowed retries.
/// * `Err(reqwest::Error)` - If all retry attempts fail, returns the last encountered error.
pub(crate) async fn fetch_url(
    url: &str,
    retry: u8,
    pause: u64,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut last_error = None;

    for attempt in 1..=retry {
        match reqwest::get(url).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                last_error = Some(e);
                eprintln!("Attempt to retry {attempt}/{retry}");
                tokio::time::sleep(tokio::time::Duration::from_secs(pause)).await;
            }
        }
    }

    Err(last_error.unwrap())
}
