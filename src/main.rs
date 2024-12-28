// ethan stoneman 2024

use clap::{command, Parser, Subcommand};
use url::Url;

mod convert;
use convert::download;

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
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Download {
            youtube_url,
            dest_type,
        } => match download(youtube_url.clone(), dest_type.as_ref().unwrap().to_string()) {
            Ok(_) => eprintln!("info: download complete"),
            Err(e) => eprintln!("error: {}", e),
        },
    }
}
