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

use serde::{Deserialize, Serialize};
// use urlencoding::encode;

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
        // youtube_id: "W8h05Soz5Nk".into(),
        youtube_id: "yPvoKz6tyJs".into(),
        quality: 5,
        format_value: 1,
    };

    // https://www.youtube.com/watch?v=yPvoKz6tyJs
    // https://www.youtube.com/watch?v=W8h05Soz5Nk
    let res_raw = client
        .post("https://cnvmp3.com/check_database.php")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&pcd)
        .send()
        .await?
        .text()
        .await?;

    let res_parsed: CNVResponse = match json_parse(&res_raw) {
        Ok(response) => response,
        Err(e) => panic!("Error parsing json: {e}"),
    };

    match res_parsed {
        CNVResponse::Data(_) => {
            // TODO: retrieve the cdn-local copy
            println!("retrieve cdn-local copy");

            // sources: [ download.php ]
        }
        CNVResponse::Error(_) => {
            // TODO: retrieve from backend server
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
