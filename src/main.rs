// ethan stoneman 2024

use clap::{command, Parser, Subcommand};
use url::Url;

mod convert;
use convert::download;
mod bitrate;
use bitrate::{BitRate, FromNumber};

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
        Commands::Download {
            youtube_url,
            dest_type,
            quality,
        } => {
            let bitrate = match quality {
                Some(q) => q,
                None => &BitRate::Kbps96,
            };

            match download(
                youtube_url.clone(),
                dest_type.as_ref().unwrap().to_string(),
                *bitrate,
            ) {
                Ok(_) => eprintln!("info: download complete"),
                Err(e) => eprintln!("error: {}", e),
            }
        }
    }
}
