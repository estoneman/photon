// ethan stoneman 20241215
//
// frontend to cnvmp3 api
//
// useful for:
//   - aspiring dj's
//   - music heads
//   - anyone with a soul
//
// but actually, this will follow the steps for downloading music from the internet after
// being converted from a youtube video to an mp3 file
//
// ╒══════╤═════════════════════╤═════════════════════╤═════════════╤══════════════════════════════╤═════════════════╕
// │ step │ url                 │ endpoint            │ http method │ payload                      │ result          │
// ├──────┼─────────────────────┼─────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
// │ 1    │ cnvmp3.com          │ /check_database.php │ POST        │ check_database.json          │ success message │
// ├──────┼─────────────────────┼─────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
// │ 2    │ cnvmp3.com          │ /get_video_data.php │ POST        │ get_video_data.json          │ success message │
// ├──────┼─────────────────────┼─────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
// │ 3    │ cnvmp3.com          │ /download_video.php │ POST        │ download_video.json          │ mp3 url         │
// ├──────┼─────────────────────┼─────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
// │ 4    │ apiv13dlp.cnvmp3.me │ /download.php       │ GET         │ N/A (query string parameter) │ mp3 file data   │
// ╘══════╧═════════════════════╧═════════════════════╧═════════════╧══════════════════════════════╧═════════════════╛

use infer::audio::is_mp3;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
struct PayloadCheckDatabase {
    youtube_id: String,
    quality: i64,
    #[serde(rename = "formatValue")]
    format_value: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckDatabaseData {
    id: i64,
    youtube_id: String,
    server_path: String,
    quality: String,
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum CNVResponse {
    Data(CheckDatabaseResponseData),
    Error(CheckDatabaseResponseError),
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckDatabaseResponseData {
    success: bool,
    data: CheckDatabaseData,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckDatabaseResponseError {
    success: bool,
    error: String,
}

fn json_parse(raw: &str) -> Result<CNVResponse, String> {
    serde_json::from_str::<CNVResponse>(raw).map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let pcd = PayloadCheckDatabase {
        youtube_id: "W8h05Soz5Nk".into(),
        // youtube_id: "yPvoKz6tyJs".into(),
        quality: 5,
        format_value: 1,
    };

    // https://www.youtube.com/watch?v=yPvoKz6tyJs
    // https://www.youtube.com/watch?v=W8h05Soz5Nk
    let cd_raw = client
        .post("https://cnvmp3.com/check_database.php")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&pcd)
        .send()
        .await?
        .text()
        .await?;

    let cd_parsed: CNVResponse = match json_parse(&cd_raw) {
        Ok(p) => p,
        Err(e) => panic!("Error parsing json: {e}"),
    };

    match cd_parsed {
        CNVResponse::Data(cd_data) => {
            // TODO: check if host-local copy exists
            let download = client
                .get(cd_data.data.server_path)
                .send()
                .await?
                .bytes()
                .await?;

            if is_mp3(&download) {
                let invalid_chars_re = Regex::new(r"[^\w\-_]").unwrap();
                let title_cleaned = invalid_chars_re.replace_all(&cd_data.data.title, "_");

                let mut outfile = File::create(format!("mp3/{title_cleaned}.mp3"))
                    .expect("file creation should succeed");
                match outfile.write_all(&download) {
                    Ok(_) => println!("{} saved successfully", title_cleaned),
                    Err(e) => println!("{:?}", e),
                }
            } else {
                let error_parsed = json_parse(std::str::from_utf8(&download).unwrap());
                let cd_error: CNVResponse = match error_parsed {
                    Ok(p) => p,
                    Err(e) => panic!("{e}"),
                };

                match cd_error {
                    CNVResponse::Error(e) => println!("{}", e.error),
                    _ => panic!("unsupported server response"),
                };
            }
        }
        CNVResponse::Error(cd_error) => {
            // TODO: retrieve from backend server
            println!("{}", cd_error.error);
            println!("retrieve from backend server");
            // sources:
            //     - get video data
            //     - download video data
            //     - insert database
            //     - download.php
        }
    };

    Ok(())
}
