// ethan stoneman 20241217
//
// frontend to cnvmp3 backend api
//
// useful for:
//   - aspiring dj's
//   - music heads
//   - anyone with a soul
//
// but actually, this will follow the steps for downloading music from the internet after
// being converted from a youtube video to an mp3 file
//
// ╒══════╤═══════════════════╤═════════════════════════╤═════════════╤══════════════════════════════╤═════════════════╕
// │ step │ url               │ endpoint                │ http method │ payload                      │ result          │
// ├──────┼───────────────────┼─────────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
// │ 1    │ cnvmp3.com        │ /check_database.php     │ POST        │ check_database.json          │ success message │
// ├──────┼───────────────────┼─────────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
// │ 2    │ cnvmp3.com        │ /get_video_data.php     │ POST        │ get_video_data.json          │ success message │
// ├──────┼───────────────────┼─────────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
// │ 3    │ cnvmp3.com        │ /download_video.php     │ POST        │ download_video.json          │ mp3 url         │
// ├──────┼───────────────────┼─────────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
// │ 4    │ cnvmp3.com        │ /insert_to_database.php │ POST        │ insert_to_database.json      │ success message │
// ├──────┼───────────────────┼─────────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
// │ 5    │ N/A (CDN-defined) │ /download.php           │ GET         │ N/A (query string parameter) │ mp3 file data   │
// ╘══════╧═══════════════════╧═════════════════════════╧═════════════╧══════════════════════════════╧═════════════════╛
//
// cli args spec:
//   youtube_url:
//     type: String
//     desc: full url to the youtube video to be converted
//     required: true

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

    if !Path::new(format!("mp3/{}.mp3", youtube_id.clone()).as_str()).exists() {
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
    }

    Ok(())
}
