use chrono::{DateTime, Utc, serde::ts_seconds};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_this_or_that::as_f64;

use super::{Endpoint, Kline};
use crate::{cli::Command, utils::split_intervals};

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
                    let _start = start.timestamp_millis();
                    let _end = end.timestamp_millis();
                    format!("{url}&from={_start}&to={_end}")
                })
                .collect::<Vec<_>>();
            return urls;
        }

        if let (Some(start), None) = (self.0.from_date, self.0.to_date) {
            url = format!("{url}&from={}", start.timestamp_millis());
        } else if let (None, Some(end)) = (self.0.from_date, self.0.to_date) {
            url = format!("{url}&to={}", end.timestamp_millis());
        } else {
            url = format!("{url}&limit=1000");
        }
        vec![url]
    }
}

/// Represents a single candlestick (kline) from Gate.
#[derive(Debug, Serialize, Deserialize)]
pub struct GateKline {
    #[serde(rename = "0", with = "ts_seconds")]
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
    window: bool,
}

impl Kline for GateKline {
    type Output = Vec<Value>;

    fn open_time(&self) -> DateTime<Utc> {
        self.time
    }

    fn close_time(&self) -> DateTime<Utc> {
        self.time
    }

    fn to_value(&self) -> Self::Output {
        vec![
            self.time.timestamp_millis().into(),
            self.quote_volume.into(),
            self.close_price.into(),
            self.high_price.into(),
            self.low_price.into(),
            self.open_price.into(),
            self.base_volume.into(),
            self.window.into(),
        ]
    }
}
