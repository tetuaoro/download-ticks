mod binance;
mod gate;

pub use binance::*;
pub use gate::*;

use chrono::{DateTime, Utc};

pub trait Kline {
    fn open_time(&self) -> DateTime<Utc>;
    fn close_time(&self) -> DateTime<Utc>;
}

pub trait Endpoint<'m> {
    fn urls(&self) -> Vec<String>;
}
