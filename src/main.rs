//! A command-line tool to fetch and save candlestick (kline) data from exchanges.
//!
//! This tool allows you to download historical candlestick data from exchanges
//! (e.g., Binance), split the time range into chunks (to respect API limits),
//! and save the results to a JSON file.
//!
//! ## Features
//! - Supports multiple exchanges.
//! - Handles large time ranges by splitting them into smaller intervals.
//! - Saves data in the original exchange format or a custom JSON structure.
//!
//! ## Usage
//! The tool is designed to be flexible and easy to use. See the `cli` module for command-line options.

mod cli;
mod errors;
mod market;
mod utils;

use futures::TryFutureExt;
use futures::{StreamExt, stream};
use indicatif::ProgressBar;
use reqwest::Client;

use crate::cli::*;
use crate::errors::*;
use crate::market::*;
use crate::utils::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = Command::build()?;
    let market: &dyn Endpoint = match cmd.market {
        Market::Gate => &Gate::build(&cmd),
        Market::Binance => &Binance::build(&cmd),
    };
    let urls = market.urls();

    let progress_bar = if cmd.verbose {
        let pb = ProgressBar::new(urls.len() as u64);
        pb.set_style(
            indicatif::ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    let client = Client::new();
    let klines_stream = stream::iter(&urls)
        .map(|url| {
            let client = &client;
            let cmd = &cmd;
            async move {
                let response = client.get(url).send().map_err(Error::from).await?;
                match cmd.market {
                    Market::Gate => {
                        response
                            .json::<Vec<GateKline>>()
                            .map_ok(|klines| klines.into_iter().map(AnyKline::Gate).collect::<Vec<_>>())
                            .map_err(Error::from)
                            .await
                    }
                    Market::Binance => {
                        response
                            .json::<Vec<BinanceKline>>()
                            .map_ok(|klines| klines.into_iter().map(AnyKline::Binance).collect::<Vec<_>>())
                            .map_err(Error::from)
                            .await
                    }
                }
            }
        })
        .buffered(90);

    let mut all_klines = Vec::with_capacity(urls.len() * 1000);
    all_klines = klines_stream
        .fold(all_klines, |mut arr, result| async {
            match result {
                Ok(klines) => {
                    arr.extend(klines);
                    if let Some(pb) = &progress_bar {
                        pb.inc(1);
                    }
                }
                Err(e) => eprintln!("{e}"),
            }
            arr
        })
        .await;

    if let Some(filepath) = cmd.output_file {
        let klines = all_klines.iter().map(|k| k.format()).collect::<Vec<_>>();
        write_to_file(filepath, &klines)?;
    }

    if cmd.verbose {
        println!("Download ticks done.");
    }

    Ok(())
}
