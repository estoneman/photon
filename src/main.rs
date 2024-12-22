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

use std::env::{args, Args};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut argv: Args = args();

    let youtube_url: Url = match argv.nth(1) {
        Some(arg) => match Url::parse(&arg) {
            Ok(u) => u,
            Err(e) => {
                eprintln!("[ERROR] unable to parse argument as a valid url: {:?}", e);
                usage();
                exit(1);
            }
        },
        None => {
            eprintln!("[ERROR] unable to get the url, was it supplied?");
            usage();
            exit(1);
        }
    };

    println!("[INFO] Using YouTube URL: {}", youtube_url);

    let youtube_id = match youtube_url.query_pairs().next() {
        Some(v) => v.1,
        None => {
            eprintln!("[ERROR] youtube url is not valid (doesn't include `v` query string key)");
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
                c.cdn_download(data.data.server_path.clone(), youtube_id.to_string())
                    .await?;
            }
            ResponseCheckDatabase::NoExist(_) => {
                let title = c.cdn_fetch(youtube_url.to_string()).await?;

                let server_path = c
                    .srv_download(youtube_url.to_string(), title.clone())
                    .await?;

                c.cdn_insert(server_path.clone(), title.clone(), youtube_id.to_string())
                    .await?;

                c.cdn_download(server_path, youtube_id.to_string()).await?;
            }
            ResponseCheckDatabase::Unknown(unknown) => {
                eprintln!("[ERROR] unknown response type: {:?}", unknown);
                exit(1);
            }
        };
    }

    Ok(())
}
