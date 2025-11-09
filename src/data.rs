use chrono::{DateTime, Utc, serde::ts_microseconds};
use serde::{Deserialize, Serialize};
use serde_this_or_that::as_f64;

mod timestamp {
    use super::*;
    use serde::de::{Error, Visitor};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TimestampVisitor;

        impl<'de> Visitor<'de> for TimestampVisitor {
            type Value = DateTime<Utc>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a timestamp in nanoseconds or milliseconds")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if let Ok(ts) = DateTime::<Utc>::from_timestamp_millis(value as i64)
                    .ok_or(E::custom("invalid milliseconds timestamp"))
                {
                    return Ok(ts);
                }

                Ok(DateTime::<Utc>::from_timestamp_nanos(value as i64))
            }
        }

        deserializer.deserialize_u64(TimestampVisitor)
    }
}

/// Represents `Kline`. Use to read from file.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Data {
    #[serde(deserialize_with = "timestamp::deserialize")]
    open_time: DateTime<Utc>,
    open_price: f64,
    high_price: f64,
    low_price: f64,
    close_price: f64,
    volume: f64,
    #[serde(deserialize_with = "timestamp::deserialize")]
    close_time: DateTime<Utc>,
    quote_asset_volume: f64,
    number_of_trades: u64,
    taker_buy_base_volume: f64,
    taker_buy_quote_volume: f64,
    ignore: f64,
}

impl Into<Kline> for Data {
    fn into(self) -> Kline {
        Kline {
            open_time: self.open_time,
            open_price: self.open_price,
            high_price: self.high_price,
            low_price: self.low_price,
            close_price: self.close_price,
            volume: self.volume,
            close_time: self.close_time,
            quote_asset_volume: self.quote_asset_volume,
            number_of_trades: self.number_of_trades,
            taker_buy_base_volume: self.taker_buy_base_volume,
            taker_buy_quote_volume: self.taker_buy_quote_volume,
            ignore: self.ignore,
        }
    }
}

/// Represents a single candlestick (kline) from Binance.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Kline {
    #[serde(rename(deserialize = "0"), with = "ts_microseconds")]
    open_time: DateTime<Utc>,
    #[serde(rename(deserialize = "1"), deserialize_with = "as_f64")]
    open_price: f64,
    #[serde(rename(deserialize = "2"), deserialize_with = "as_f64")]
    high_price: f64,
    #[serde(rename(deserialize = "3"), deserialize_with = "as_f64")]
    low_price: f64,
    #[serde(rename(deserialize = "4"), deserialize_with = "as_f64")]
    close_price: f64,
    #[serde(rename(deserialize = "5"), deserialize_with = "as_f64")]
    volume: f64,
    #[serde(rename(deserialize = "6"), with = "ts_microseconds")]
    close_time: DateTime<Utc>,
    #[serde(rename(deserialize = "7"), deserialize_with = "as_f64")]
    quote_asset_volume: f64,
    #[serde(rename(deserialize = "8"))]
    number_of_trades: u64,
    #[serde(rename(deserialize = "9"), deserialize_with = "as_f64")]
    taker_buy_base_volume: f64,
    #[serde(rename(deserialize = "10"), deserialize_with = "as_f64")]
    taker_buy_quote_volume: f64,
    #[serde(rename(deserialize = "11"), deserialize_with = "as_f64")]
    ignore: f64,
}

impl Kline {
    pub(crate) fn close_time(&self) -> DateTime<Utc> {
        self.close_time
    }
}
