// ethan stoneman 2024

use clap::{command, Parser, Subcommand};
use std::ops::RangeInclusive;
use std::path::Path;
use std::process::exit;
use url::Url;

mod client;
use client::*;

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

/// Converts a YouTube video to an MP3 file and downloads it.
///
/// # Arguments
///
/// * `youtube_url` - The URL of the YouTube video to convert.
///
/// # Returns
///
/// * `Ok(())` - If the MP3 file is downloaded successfully.
/// * `Err` - If an error occurs during conversion or download.
///
/// # Example
///
/// ```rust
/// use url::Url;
///
/// #[tokio::main]
/// async fn main() {
///     let youtube_url = Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
///     if let Err(e) = convert(youtube_url).await {
///         eprintln!("Error: {}", e);
///     } else {
///         println!("Download successful!");
///     }
/// }
/// ```
///
/// # Notes
///
/// This function uses `cnvmp3.com` to perform the conversion.
async fn download(youtube_url: Url) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(youtube_url.host_str(), Some("www.youtube.com"));

    let youtube_id: String = match youtube_url.query_pairs().next() {
        Some(v) => v.1.to_string(),
        None => {
            eprintln!(
                "error: youtube url is not valid (doesn't include youtube id in query string)"
            );
            exit(1);
        }
    };

    if Path::new(format!("mp3/{}.mp3", youtube_id.clone()).as_str()).exists() {
        println!("the requested video has already been saved locally as mp3");
    }

    let c = CNVClient {
        client: reqwest::Client::new(),
    };

    let checkdb_res = c.check_database(youtube_id.to_string()).await?;

    match match_response(checkdb_res) {
        ResponseCheckDatabase::Exist(data) => {
            eprintln!("info: file exists in cdn");
            c.cdn_download(data.data.server_path.clone(), youtube_id.to_string())
                .await?;
        }
        ResponseCheckDatabase::NoExist(_) => {
            eprintln!("info: file does not exist in cdn");
            let title = c.cdn_fetch(youtube_url.to_string()).await?;

            let server_path = c
                .srv_download(youtube_url.to_string(), title.clone())
                .await?;

            c.cdn_insert(server_path.clone(), title.clone(), youtube_id.to_string())
                .await?;

            c.cdn_download(server_path, youtube_id.to_string()).await?;
        }
        ResponseCheckDatabase::Unknown(unknown) => {
            eprintln!("error: unknown response type: {:?}", unknown);
            exit(1);
        }
    };

    Ok(())
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        // TODO: use the dest type when downloading
        #[allow(unused_variables)]
        Commands::Download {
            youtube_url,
            dest_type,
        } => {
            download(youtube_url.clone()).await?;
        }
        Commands::Analysis { bpm } => {
            analysis(*bpm);
        }
    }

    Ok(())
}
