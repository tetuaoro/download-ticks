use chrono::{DateTime, Utc, serde::ts_milliseconds};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_this_or_that::{as_f64, as_u64};

use super::{Endpoint, Kline};
use crate::{cli::Command, utils::split_intervals};

pub struct Binance<'b>(&'b Command);

impl<'b> Binance<'b> {
    const BASE_URL: &'b str = "https://api.binance.com/api/v3/klines";

    pub fn build(command: &'b Command) -> Self {
        Self(command)
    }
}

impl<'b> Endpoint<'b> for Binance<'b> {
    fn urls(&self) -> Vec<String> {
        let symbol = &self.0.symbol;
        let interval = &self.0.interval;
        let burl = Self::BASE_URL;
        let mut url = format!("{burl}?symbol={symbol}&interval={interval}&limit=1000");

        if let (Some(start), Some(end)) = (self.0.from_date, self.0.to_date) {
            let datetimes = split_intervals(start, end, interval);
            let urls = datetimes
                .iter()
                .map(|(start, end)| {
                    let _start = start.timestamp_millis();
                    let _end = end.timestamp_millis();
                    format!("{url}&startTime={_start}&endTime={_end}")
                })
                .collect::<Vec<_>>();
            return urls;
        }

        if let (Some(start), None) = (self.0.from_date, self.0.to_date) {
            url = format!("{url}&startTime={}", start.timestamp_millis());
        } else if let (None, Some(end)) = (self.0.from_date, self.0.to_date) {
            url = format!("{url}&endTime={}", end.timestamp_millis());
        }
        vec![url]
    }
}

/// Represents a single candlestick (kline) from binance.
#[derive(Debug, Serialize, Deserialize)]
pub struct BinanceKline {
    #[serde(rename = "0", with = "ts_milliseconds")]
    open_time: DateTime<Utc>,
    #[serde(rename = "1", deserialize_with = "as_f64")]
    open_price: f64,
    #[serde(rename = "2", deserialize_with = "as_f64")]
    high_price: f64,
    #[serde(rename = "3", deserialize_with = "as_f64")]
    low_price: f64,
    #[serde(rename = "4", deserialize_with = "as_f64")]
    close_price: f64,
    #[serde(rename = "5", deserialize_with = "as_f64")]
    volume: f64,
    #[serde(rename = "6", with = "ts_milliseconds")]
    close_time: DateTime<Utc>,
    #[serde(rename = "7", deserialize_with = "as_f64")]
    quote_asset_volume: f64,
    #[serde(rename = "8")]
    number_of_trades: u64,
    #[serde(rename = "9", deserialize_with = "as_f64")]
    taker_buy_base_volume: f64,
    #[serde(rename = "10", deserialize_with = "as_f64")]
    taker_buy_quote_volume: f64,
    #[serde(rename = "11", deserialize_with = "as_u64")]
    ignore: u64,
}

impl Kline for BinanceKline {
    fn close_time(&self) -> DateTime<Utc> {
        self.close_time
    }

    fn format(&self) -> Vec<Value> {
        vec![
            self.open_time.timestamp_millis().into(),
            self.open_price.into(),
            self.high_price.into(),
            self.low_price.into(),
            self.close_price.into(),
            self.volume.into(),
            self.close_time.timestamp_millis().into(),
            self.quote_asset_volume.into(),
            self.number_of_trades.into(),
            self.taker_buy_base_volume.into(),
            self.taker_buy_quote_volume.into(),
            self.ignore.into(),
        ]
    }
}
