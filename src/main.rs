// ethan stoneman 2024

use clap::{command, Parser};
use std::path::Path;
use std::process::exit;
use url::Url;

mod client;
use client::*;

fn usage() {
    eprintln!(
        "usage:\n  \
photon <youtube-url>\n\
example:\n  \
photon https://www.youtube.com/watch?v=shus67s72"
    )
}

/// Command-line argument specification
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// A valid YouTube URL
    #[arg(long, value_name = "URL")]
    youtube_url: String,
    /// Where to store the returned MP3 file
    #[arg(long, value_parser = ["local", "ssh"], value_name = "TYPE", default_value = "local")]
    dest_type: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let youtube_url: Url = match Url::parse(&args.youtube_url) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("error: Unable to parse argument as a valid url: {:?}", e);
            usage();
            exit(1);
        }
    };

    assert_eq!(youtube_url.host_str(), Some("www.youtube.com"));
    println!("info: Using YouTube URL: {}", youtube_url);

    let youtube_id = match youtube_url.query_pairs().next() {
        Some(v) => v.1,
        None => {
            eprintln!("error: youtube url is not valid (doesn't include `v` query string key)");
            usage();
            exit(1);
        }
    };

    if Path::new(format!("mp3/{}.mp3", youtube_id.clone()).as_str()).exists() {
        println!("the requested video has already been saved locally as mp3");
        return Ok(());
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
