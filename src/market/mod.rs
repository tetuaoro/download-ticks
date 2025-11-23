mod binance;
mod gate;

pub use binance::*;
pub use gate::*;

use chrono::{DateTime, Utc};
use serde_json::{Value, to_value};

pub trait Kline {
    type Output;

    fn open_time(&self) -> DateTime<Utc>;
    fn close_time(&self) -> DateTime<Utc>;
    fn to_value(&self) -> Self::Output;
}

pub trait Endpoint<'m> {
    fn urls(&self) -> Vec<String>;
}

pub enum AnyKline {
    Gate(GateKline),
    Binance(BinanceKline),
}

impl Kline for AnyKline {
    type Output = Value;

    fn open_time(&self) -> DateTime<Utc> {
        match self {
            AnyKline::Gate(k) => k.open_time(),
            AnyKline::Binance(k) => k.open_time(),
        }
    }

    fn close_time(&self) -> DateTime<Utc> {
        match self {
            AnyKline::Gate(k) => k.close_time(),
            AnyKline::Binance(k) => k.close_time(),
        }
    }

    fn to_value(&self) -> Self::Output {
        match self {
            AnyKline::Gate(k) => to_value(k.to_value()).unwrap(),
            AnyKline::Binance(k) => to_value(k.to_value()).unwrap(),
        }
    }
}
