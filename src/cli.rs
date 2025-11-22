use std::fmt;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use clap::{Parser, ValueEnum};

use crate::errors::{Error, Result};

/// Supported market to fetch the data.
#[derive(Debug, Clone, ValueEnum)]
pub enum Market {
    Binance,
}

impl fmt::Display for Market {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Market::Binance => write!(f, "binance"),
        }
    }
}

/// Supported time intervals for klines.
#[derive(Debug, Clone, ValueEnum)]
pub enum Interval {
    /// 1 second
    S1,
    /// 1 minute
    M1,
    /// 3 minutes
    M3,
    /// 5 minutes
    M5,
    /// 15 minutes
    M15,
    /// 30 minutes
    M30,
    /// 1 hour
    H1,
    /// 2 hours
    H2,
    /// 4 hours
    H4,
    /// 6 hours
    H6,
    /// 8 hours
    H8,
    /// 12 hours
    H12,
    /// 1 day
    D1,
    /// 3 days
    D3,
    /// 1 week
    W1,
    /// 1 month
    MM1,
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Interval::S1 => write!(f, "1s"),
            Interval::M1 => write!(f, "1m"),
            Interval::M3 => write!(f, "3m"),
            Interval::M5 => write!(f, "5m"),
            Interval::M15 => write!(f, "15m"),
            Interval::M30 => write!(f, "30m"),
            Interval::H1 => write!(f, "1h"),
            Interval::H2 => write!(f, "2h"),
            Interval::H4 => write!(f, "4h"),
            Interval::H6 => write!(f, "6h"),
            Interval::H8 => write!(f, "8h"),
            Interval::H12 => write!(f, "12h"),
            Interval::D1 => write!(f, "1d"),
            Interval::D3 => write!(f, "3d"),
            Interval::W1 => write!(f, "1w"),
            Interval::MM1 => write!(f, "1M"),
        }
    }
}

/// Command-line arguments for the program.
#[derive(Debug, Clone, Parser)]
#[command(
    version,
    about,
    long_about = "
This tool fetches historical candlestick (kline) data from exchanges like Binance.
You can specify a trading pair (e.g., BTCUSDT), time interval (e.g., 1m, 1h, 1d),
and optional start/end dates. The data is saved to a JSON file if --output-file is provided.

## Supported Exchanges
- Binance

## Supported Intervals
- Seconds: 1s
- Minutes: 1m, 3m, 5m, 15m, 30m
- Hours: 1h, 2h, 4h, 6h, 8h, 12h
- Days: 1d, 3d
- Weeks: 1w
- Months: 1M

Examples:
  Fetch 1-hour BTCUSDT klines for the last 1000 hours:
    $ download-ticks -s BTCUSDT -i h1

  Fetch 1-minute BTCUSDT klines from Jan 1, 2019, to Mar 1, 2019, and save to output.json:
    $ download-ticks -s BTCUSDT -i m1 --from-date '2019-01-01T00:00:00Z' --to-date '2019-03-01T00:00:00Z' -o output.json
"
)]
pub struct Command {
    /// The market to fetch the data (e.g., binance).
    #[arg(short, long, default_value_t = Market::Binance)]
    pub market: Market,

    /// The trading pair symbol (e.g., BTCUSDT, ETHUSDT).
    #[arg(short, long)]
    pub symbol: String,

    /// The time interval for klines.
    #[arg(short, long)]
    pub interval: Interval,

    /// Start date for fetching klines (UTC, RFC 3339 format).
    #[arg(short, long)]
    pub from_date: Option<DateTime<Utc>>,

    /// End date for fetching klines (UTC, RFC 3339 format).
    #[arg(short, long)]
    pub to_date: Option<DateTime<Utc>>,

    /// Output file path to save the klines in JSON format.
    #[arg(short, long)]
    pub output_file: Option<PathBuf>,
    // /// Re-try to get ticks from marketplace.
    // #[arg(short, long, default_value = "3")]
    // pub retry_counter: u8,

    // /// Print progress status. Usefull if you get `from` and `to` dates.
    // #[arg(short, long)]
    // pub verbose: bool,
}

impl Command {
    pub fn build() -> Result<Self> {
        let cmd = Self::parse();
        if let (Some(from_date), Some(to_date)) = (cmd.from_date, cmd.to_date) {
            if to_date < from_date {
                return Err(Error::InvalidDatetime);
            }
        }
        Ok(cmd)
    }
}
