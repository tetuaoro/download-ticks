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
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

use crate::cli::*;
use crate::errors::*;
use crate::market::*;
use crate::utils::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::build()?;
    match &cli.command {
        Commands::Info(cmd) => {
            let filepath = &cmd.input_file;
            if let Ok(data) = read_data_from_file::<BinanceKline>(filepath) {
                let first_k = data.first().ok_or(Error::MissingData)?;
                let first_open_time = first_k.open_time();
                let last_k = data.last().ok_or(Error::MissingData)?;
                let last_close_time = last_k.close_time();
                let n = data.len();

                println!(
                    "
========================

Number of elements: {n}

It was started from {first_open_time},
and ended to {last_close_time}.

========================
"
                );
            } else {
                let data = read_data_from_file::<GateKline>(filepath).map_err(|_| Error::InvalidFile)?;
                let first_k = data.first().ok_or(Error::MissingData)?;
                let first_open_time = first_k.open_time();
                let last_k = data.last().ok_or(Error::MissingData)?;
                let last_close_time = last_k.close_time();
                let n = data.len();

                println!(
                    "
========================

Number of elements: {n}

It was started from {first_open_time},
and ended to {last_close_time}.

========================
"
                );
            }
        }
        Commands::Fetch(cmd) => {
            let market: &dyn Endpoint = match cmd.market {
                Market::Gate => &Gate::build(&cmd),
                Market::Binance => &Binance::build(&cmd),
            };
            let urls = market.urls();

            let progress_bar = if cmd.verbose {
                let pb = ProgressBar::new(urls.len() as u64);
                pb.set_style(
                    ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})\n{msg}")?
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
                    let progress_bar = &progress_bar;
                    async move {
                        let response = client.get(url).send().map_err(Error::from).await?;
                        if let Some(pb) = progress_bar {
                            pb.inc(1);
                        }
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
                            if let Some(pb) = &progress_bar {
                                pb.inc(1);
                            }
                            arr.extend(klines);
                        }
                        Err(e) => {
                            if let Some(pb) = &progress_bar {
                                pb.abandon_with_message(e.to_string());
                            } else {
                                eprintln!("{e}");
                            }
                        }
                    }
                    arr
                })
                .await;

            if let Some(filepath) = &cmd.output_file {
                let klines = all_klines.iter().map(|k| k.to_value()).collect::<Vec<_>>();
                write_to_file(filepath, &klines)?;
            }
            if let Some(pb) = &progress_bar {
                pb.finish_with_message("Download ticks done.");
            } else {
                println!("Download ticks done.");
            }
        }
    };

    Ok(())
}
