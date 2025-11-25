#![allow(unused)]

use chrono::{DateTime, Utc, serde::ts_seconds};
use serde::{Deserialize, de::Error as DeError};
use serde_this_or_that::{as_bool, as_f64};

use super::{Endpoint, Kline};
use crate::{cli::Command, errors::Error, utils::split_intervals};

/// A wrapper for the Gate.io exchange configuration.
pub struct Gate<'b>(&'b Command);

impl<'b> Gate<'b> {
    const BASE_URL: &'b str = "https://api.gateio.ws/api/v4/spot/candlesticks";

    pub fn build(command: &'b Command) -> Self {
        Self(command)
    }
}

impl<'b> Endpoint<'b> for Gate<'b> {
    fn urls(&self) -> Vec<String> {
        let symbol = &self.0.symbol;
        let interval = &self.0.interval;
        let burl = Self::BASE_URL;
        let mut url = format!("{burl}?currency_pair={symbol}&interval={interval}");

        if let (Some(start), Some(end)) = (self.0.from_date, self.0.to_date) {
            let datetimes = split_intervals(start, end, interval);
            let urls = datetimes
                .iter()
                .map(|(start, end)| {
                    let _start = start.timestamp();
                    let _end = end.timestamp();
                    format!("{url}&from={_start}&to={_end}")
                })
                .collect::<Vec<_>>();
            return urls;
        }

        if let (Some(start), None) = (self.0.from_date, self.0.to_date) {
            url = format!("{url}&from={}", start.timestamp());
        } else if let (None, Some(end)) = (self.0.from_date, self.0.to_date) {
            url = format!("{url}&to={}", end.timestamp());
        } else {
            url = format!("{url}&limit=1000");
        }
        vec![url]
    }
}

/// Represents a single candlestick (kline) from Gate.
#[derive(Debug, Deserialize)]
pub struct GateKline {
    #[serde(rename = "0", deserialize_with = "to_datetime")]
    time: DateTime<Utc>,
    #[serde(rename = "1", deserialize_with = "as_f64")]
    quote_volume: f64,
    #[serde(rename = "2", deserialize_with = "as_f64")]
    close_price: f64,
    #[serde(rename = "3", deserialize_with = "as_f64")]
    high_price: f64,
    #[serde(rename = "4", deserialize_with = "as_f64")]
    low_price: f64,
    #[serde(rename = "5", deserialize_with = "as_f64")]
    open_price: f64,
    #[serde(rename = "6", deserialize_with = "as_f64")]
    base_volume: f64,
    #[serde(rename = "6", deserialize_with = "as_bool")]
    window: bool,
}

impl Kline for GateKline {
    fn open_time(&self) -> DateTime<Utc> {
        self.time
    }

    fn close_time(&self) -> DateTime<Utc> {
        self.time
    }
}

/// Deserializes a string into a `DateTime<Utc>`.
///
/// This function supports timestamps in seconds.
///
/// # Errors
/// Returns an error if the string cannot be parsed as an integer or if the timestamp is invalid.
pub fn to_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let deserialize = String::deserialize(deserializer)?;
    let timestamp = deserialize.parse::<i64>().map_err(DeError::custom)?;
    DateTime::<Utc>::from_timestamp_secs(timestamp).ok_or_else(|| DeError::custom(Error::InvalidDatetime))
}
