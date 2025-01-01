// ethan stoneman 2024

use clap::{command, Parser, Subcommand};
use url::Url;

mod bitrate;
mod convert;
mod error;
mod youtube_url;

use bitrate::{BitRate, FromNumber};
use convert::y2mp3;

/// Top-level command-line argument specification
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Currently supported subcommands
#[derive(Subcommand)]
enum Commands {
    /// Converts YouTube videos to local mp3 files
    Y2Mp3 {
        /// The bitrate at which to download the MP3 file
        #[arg(long, value_parser = bitrate_parser, value_name = "BITRATE")]
        quality: Option<BitRate>,
        /// Where to store the returned MP3 file
        #[arg(long, value_parser = ["local", "ssh"], value_name = "TYPE", default_value = "local")]
        dest_type: Option<String>,
        /// A valid YouTube URL
        #[arg(long, value_name = "URL")]
        youtube_url: Url,
    },
}

fn bitrate_parser(s: &str) -> Result<BitRate, String> {
    let bitrate: u16 = s.parse().map_err(|_| format!("`{s}` is not a number"))?;

    match BitRate::from_number(bitrate) {
        Ok(b) => Ok(b),
        Err(e) => Err(format!("{e:?}")),
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Y2Mp3 {
            youtube_url,
            dest_type,
            quality,
        } => {
            let bitrate: BitRate = match quality {
                Some(q) => *q,
                None => BitRate::Kbps96,
            };

            match y2mp3(
                youtube_url.clone(),
                dest_type.as_ref().unwrap().to_string(),
                bitrate,
            ) {
                Ok(_) => eprintln!("info: conversion complete"),
                Err(e) => eprintln!("error: {}", e),
            }
        }
    }
}
