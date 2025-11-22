mod binance;
mod gate;

pub use binance::*;
pub use gate::*;

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

pub enum AnyKline {
    Gate(GateKline),
    Binance(BinanceKline),
}

impl Kline for AnyKline {
    fn close_time(&self) -> DateTime<Utc> {
        match self {
            AnyKline::Gate(gate_kline) => gate_kline.close_time(),
            AnyKline::Binance(binance_kline) => binance_kline.close_time(),
        }
    }

    fn format(&self) -> Vec<Value> {
        match self {
            AnyKline::Gate(gate_kline) => gate_kline.format(),
            AnyKline::Binance(binance_kline) => binance_kline.format(),
        }
    }
}
