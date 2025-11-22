mod binance;

pub use binance::*;

use chrono::{DateTime, Utc};
use serde_json::Value;

pub trait Kline {
    #[allow(unused)]
    fn close_time(&self) -> DateTime<Utc>;
    fn format(&self) -> Vec<Value>;
}

pub trait Endpoint<'m> {
    fn urls(&self) -> Vec<String>;
}
