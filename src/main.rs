// ethan stoneman 2024

use clap::{command, Parser, Subcommand};
use std::ops::RangeInclusive;
use url::Url;

mod convert;
use convert::*;

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
    Download {
        /// A valid YouTube URL
        #[arg(long, value_name = "URL")]
        youtube_url: Url,
        /// Where to store the returned MP3 file
        #[arg(long, value_parser = ["local", "ssh"], value_name = "TYPE", default_value = "local")]
        dest_type: Option<String>,
    },
    /// Shows rekordbox track analysis information
    Analysis {
        /// Positive value for Beats Per Minute (BPM)
        #[arg(long, value_name = "BPM", value_parser = bpm_in_range)]
        bpm: Option<u8>,
    },
}

const BPM_RANGE: RangeInclusive<u8> = 120..=140;

fn bpm_in_range(s: &str) -> Result<u8, String> {
    let bpm: u8 = s.parse().map_err(|_| {
        format!(
            "`{s}` isn't a sane bpm value\ntry something in this range {}-{}",
            BPM_RANGE.start(),
            BPM_RANGE.end()
        )
    })?;
    if BPM_RANGE.contains(&bpm) {
        Ok(bpm)
    } else {
        Err(format!(
            "bpm not in range {}-{}",
            BPM_RANGE.start(),
            BPM_RANGE.end()
        ))
    }
}

fn analysis(bpm: Option<u8>) {
    match bpm {
        Some(bpm) => eprintln!("info: filtering tracks with bpm at {}", bpm),
        None => eprintln!("info: not filtering tracks with a certain bpm value"),
    }
    eprintln!("this code will not run, it is not finished");
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        // TODO: use the dest type when downloading
        #[allow(unused_variables)]
        Commands::Download {
            youtube_url,
            dest_type,
        } => match download(youtube_url.clone()) {
            Ok(_) => eprintln!("info: download complete"),
            Err(e) => eprintln!("{:?}", e),
        },
        Commands::Analysis { bpm } => {
            analysis(*bpm);
        }
    }
}
