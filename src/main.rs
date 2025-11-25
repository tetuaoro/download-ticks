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
use serde_json::Value;

use crate::cli::*;
use crate::errors::*;
use crate::market::*;
use crate::utils::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::build()?;
    match &cli.command {
        Commands::Info(command) => info(&command)?,
        Commands::Fetch(command) => fetch(&command).await?,
    }

    Ok(())
}

/// Prints information about a collection of klines.
///
/// This function prints the number of elements, the start time, and the end time of the klines.
///
/// # Arguments
/// * `data` - A slice of klines implementing the `Kline` trait.
///
/// # Errors
/// Returns an error if the data slice is empty.
fn print_info<T: Kline>(data: &[T]) -> Result<()> {
    let first_k = data.first().ok_or(Error::MissingData)?;
    let last_k = data.last().ok_or(Error::MissingData)?;
    let n = data.len();
    let duration = last_k.close_time() - first_k.open_time();

    let days = duration.num_days();
    let hours = duration.num_hours();
    let minutes = duration.num_minutes();

    let duration = if minutes > 0 {
        format!("{}D / {}H / {}m", separator(days, "_")?, separator(hours, "_")?, separator(minutes, "_")?)
    } else if hours > 0 {
        format!("{}D / {}H", separator(days, "_")?, separator(hours, "_")?)
    } else {
        format!("{}D", separator(days, "_")?)
    };

    println!(
        "
========================
Number of elements: {n}
Duration: {duration}
It started from {open_time},
and ended at {close_time}.
========================
",
        open_time = first_k.open_time(),
        close_time = last_k.close_time()
    );

    Ok(())
}

/// Displays information about a JSON file containing klines.
///
/// This function reads the file, parses the klines, and prints information such as the number of elements,
/// the start time, and the end time.
///
/// # Arguments
/// * `cmd` - A reference to the info command configuration.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
fn info(cmd: &InfoCommand) -> Result<()> {
    let filepath = &cmd.input_file;
    if let Ok(data) = read_data_from_file::<BinanceKline>(filepath) {
        return print_info(&data);
    }
    if let Ok(data) = read_data_from_file::<GateKline>(filepath) {
        return print_info(&data);
    }

    Err(Error::InvalidFile)
}

/// Fetches klines data from the specified exchange.
///
/// # Arguments
/// * `cmd` - A reference to the command configuration.
///
/// # Errors
/// Returns an error if the fetch operation fails.
async fn fetch(cmd: &Command) -> Result<()> {
    let market: &dyn Endpoint = match cmd.market {
        Market::Gate => &Gate::build(&cmd),
        Market::Binance => &Binance::build(&cmd),
    };
    let urls = market.urls();

    println!("{}", urls[0]);

    let progress_bar = if cmd.verbose {
        let pb = ProgressBar::new(urls.len() as u64);
        let style = ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})\n{msg}")
            .map_err(Error::from)?
            .progress_chars("#>-");
        pb.set_style(style);
        Some(pb)
    } else {
        None
    };

    let client = Client::new();
    let klines_stream = stream::iter(&urls)
        .map(|url| {
            let client = &client;
            async move {
                let response = client.get(url).send().map_err(Error::from).await?;
                response.json::<Vec<Value>>().map_err(Error::from).await
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
                    }
                }
            }
            arr
        })
        .await;

    if let Some(filepath) = &cmd.output_file {
        write_to_file(filepath, &all_klines)?;
    }

    if let Some(pb) = &progress_bar {
        pb.finish_with_message("Download ticks done.");
    } else {
        println!("Download ticks done.");
    }

    Ok(())
}
