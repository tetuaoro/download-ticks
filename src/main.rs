use std::{fs::File, path::PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc, serde::ts_microseconds};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::to_writer;
use serde_this_or_that::as_f64;

#[derive(Debug, Clone, ValueEnum)]
enum Interval {
    /// 1 minute
    M1,
    /// 1 hour
    H1,
    /// 1 day
    D1,
}

#[derive(Debug, Parser)]
#[command(version, about)]
struct Command {
    #[arg(short, long)]
    symbol: String,
    #[arg(short, long)]
    interval: Interval,
    #[arg(short, long)]
    from_date: Option<DateTime<Utc>>,
    #[arg(short, long)]
    to_date: Option<DateTime<Utc>>,
    #[arg(short, long)]
    output_file: Option<PathBuf>,
}

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

    let url = format!("{BASE_URL}?{symbol}&{interval}");
    let response = reqwest::get(url).await?;
    let klines = response.json::<Vec<Kline>>().await?;

    if let Some(path) = cmd.output_file {
        let file = File::create(&path)?;
        to_writer(file, &klines)?;
    }

    Ok(())
}
