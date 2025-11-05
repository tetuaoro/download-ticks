//! A command-line tool to fetch and save Binance kline (candlestick) data.
//!
//! This tool allows you to download historical candlestick data from Binance,
//! split the time range into chunks (to respect Binance's 1000-candle limit),
//! and save the results to a JSON file.

mod utils;

use std::path::PathBuf;

use anyhow::Result;
use chrono::{DateTime, Utc, serde::ts_microseconds};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_this_or_that::as_f64;

use crate::utils::*;

/// Supported time intervals for Binance klines.
#[derive(Debug, Clone, ValueEnum)]
enum Interval {
    /// 1 minute
    M1,
    /// 1 hour
    H1,
    /// 1 day
    D1,
}

/// Command-line arguments for the program.
#[derive(Debug, Clone, Parser)]
#[command(
    version,
    about,
    long_about = "
This tool fetches historical candlestick data from Binance's API.
You can specify a symbol (e.g., BTCUSDT), time interval (1m, 1h, 1d),
and optional start/end dates. The data is saved to a JSON file if --output-file is provided.

Examples:
  Fetch 1-hour BTCUSDT klines for the last 1000 hours:
    $ download-ticks -s BTCUSDT -i h1

  Fetch 1-minute BTCUSDT klines from Jan 1, 2019, to Mar 1, 2019, and save to output.json:
    $ download-ticks -s BTCUSDT -i m1 --from-date '2019-01-01T00:00:00Z' --to-date '2019-03-01T00:00:00Z' -o output.json
"
)]
struct Command {
    /// The trading pair symbol (e.g., BTCUSDT, ETHUSDT).
    #[arg(short, long)]
    symbol: String,

    /// The time interval for klines (1m, 1h, 1d).
    #[arg(short, long)]
    interval: Interval,

    /// Start date for fetching klines (UTC, RFC 3339 format).
    #[arg(short, long)]
    from_date: Option<DateTime<Utc>>,

    /// End date for fetching klines (UTC, RFC 3339 format).
    #[arg(short, long)]
    to_date: Option<DateTime<Utc>>,

    /// Output file path to save the klines in JSON format.
    #[arg(short, long)]
    output_file: Option<PathBuf>,

    /// Print progress status.Usefull if you get `from` and `to` dates.
    #[arg(short, long)]
    verbose: bool,
}

/// Represents a single candlestick (kline) from Binance.
#[derive(Debug, Serialize, Deserialize)]
struct Kline {
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

/// Base URL for API endpoint.
const BASE_URL: &str = "https://api.binance.com/api/v3/klines";

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = Command::parse();

    let symbol = format!("symbol={}", cmd.symbol);
    let interval = match cmd.interval {
        Interval::M1 => "interval=1m",
        Interval::H1 => "interval=1h",
        Interval::D1 => "interval=1d",
    };

    let mut url = format!("{BASE_URL}?{symbol}&{interval}");
    let mut klines = vec![];

    if let (Some(start), Some(end)) = (cmd.from_date, cmd.to_date) {
        let url_cloned = url.clone();
        let intervals = split_intervals(start, end, &cmd.interval);

        let intervals_len = intervals.len();
        let mut progress = 1;

        for (start, end) in intervals {
            url = format!(
                "{url_cloned}&startTime={}&endTime={}",
                start.timestamp_micros(),
                end.timestamp_micros()
            );

            let response = reqwest::get(url).await?;
            let mut data = response.json::<Vec<Kline>>().await?;
            klines.append(&mut data);

            let _cmd = cmd.clone();

            if let Some(path) = _cmd.output_file {
                if let Err(e) = write_data_to_file(path, &klines) {
                    eprintln!("{e}");
                }
            }

            if _cmd.verbose {
                let percent = progress as f64 * 100.0 / intervals_len as f64;
                println!("{progress}/{intervals_len} ({percent:.3}%)");
                progress += 1;
            }
        }
    } else {
        let response = reqwest::get(url).await?;
        let mut data = response.json::<Vec<Kline>>().await?;
        klines.append(&mut data);
    }

    if let Some(path) = cmd.output_file {
        if let Err(e) = write_data_to_file(path, &klines) {
            eprintln!("{e}");
        }
    }

    Ok(())
}
