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
//
// ========
// PAYLOADS
// ========
//
// let videoData = {
//     check_database: {
//         youtube_id: youtubeId,
//         quality: formatValue === 1 ? audioQuality : videoQuality,
//     },
//     get_video_data: {
//         url: videoUrl
//     },
//     download_video: {
//         url: videoUrl,
//         quality: formatValue === 1 ? audioQuality : videoQuality,
//         title: videoTitle,
//         formatValue: formatValue,
//     },
//     insert_to_database: {
//         youtube_id: youtubeId,
//         server_path: videoServerPath,
//         quality: formatValue === 1 ? audioQuality : videoQuality,
//         title: videoTitle,
//         formatValue: formatValue,
//     }
// }

use infer::audio::is_mp3;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env::{args, Args};
use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::process::exit;
use url::Url;

// check_database.php
#[derive(Debug, Serialize)]
struct PayloadCheckDatabase {
    youtube_id: String,
    quality: i64,
    #[serde(rename = "formatValue")]
    format_value: i64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ResponseCheckDatabaseData {
    id: i64,
    youtube_id: String,
    server_path: String,
    quality: String,
    title: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ResponseCheckDatabaseExist {
    success: bool,
    data: ResponseCheckDatabaseData,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ResponseCheckDatabaseNoExist {
    success: bool,
    error: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)] // Automatically pick the correct variant based on the JSON structure
enum ResponseCheckDatabase {
    Exist(ResponseCheckDatabaseExist),
    NoExist(ResponseCheckDatabaseNoExist),
    Unknown(Value),
}

// get_video_data.php
#[derive(Debug, Serialize)]
struct PayloadGetVideoData {
    url: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ResponseGetVideoData {
    success: bool,
    title: String,
}

// download_video.php
#[derive(Debug, Serialize)]
struct PayloadDownloadVideo {
    #[serde(rename = "formatValue")]
    format_value: i64,
    url: String,
    quality: i64,
    title: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ResponseDownloadVideo {
    success: bool,
    download_link: String,
}

// insert_database.php
#[derive(Debug, Serialize, Deserialize)]
struct PayloadInsertDatabase {
    #[serde(rename = "formatValue")]
    format_value: i64,
    youtube_id: String,
    server_path: String,
    quality: i64,
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseInsertDatabase {
    success: bool,
    message: String,
}

fn usage() {
    eprintln!(
        "usage: \
  y2mp3 <youtube-url> \
\
  e.g., y2mp3 https://www.youtube.com/watch?v=shus67s72"
    )
}

fn match_response(value: Value) -> ResponseCheckDatabase {
    // Check if it matches the structure of `SuccessData`
    if let Ok(data) = serde_json::from_value::<ResponseCheckDatabaseExist>(value.clone()) {
        let some_data: ResponseCheckDatabaseExist = data;
        return ResponseCheckDatabase::Exist(some_data);
    }

    // Check if it matches the structure of `ErrorData`
    if let Ok(error) = serde_json::from_value::<ResponseCheckDatabaseNoExist>(value.clone()) {
        let some_error: ResponseCheckDatabaseNoExist = error;
        return ResponseCheckDatabase::NoExist(some_error);
    }

    // Fallback for unknown structures
    ResponseCheckDatabase::Unknown(value)
}

fn json_parse<T>(raw: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    serde_json::from_str::<T>(raw).map_err(|e| e.to_string())
}

async fn cdn_download(
    client: reqwest::Client,
    server_path: String,
    title: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let download = client.get(server_path).send().await?.bytes().await?;

    if is_mp3(&download) {
        let mut outfile =
            File::create(format!("mp3/{}.mp3", title)).expect("file creation should succeed");

        match outfile.write_all(&download) {
            Ok(_) => println!("{} saved successfully", title),
            Err(e) => println!("{:?}", e),
        }
    } else {
        println!(
            "downloaded content is not an mp3 file:\n{}",
            std::str::from_utf8(&download).unwrap()
        );
    }

    Ok(())
}

async fn cdn_fetch(
    client: reqwest::Client,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let pgvd = PayloadGetVideoData {
        url: url.to_string(),
    };

    let gvd_res_text = client
        .post("https://cnvmp3.com/get_video_data.php")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&pgvd)
        .send()
        .await?
        .text()
        .await?;

    let gvd_res_parsed: ResponseGetVideoData = match json_parse(&gvd_res_text) {
        Ok(p) => p,
        Err(e) => panic!("Error parsing json: {e}"),
    };

    Ok(gvd_res_parsed.title)
}

async fn cdn_insert(
    client: reqwest::Client,
    server_path: String,
    title: String,
    youtube_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let pid = PayloadInsertDatabase {
        format_value: 1,
        quality: 5,
        youtube_id: youtube_id,
        server_path: server_path,
        title: title,
    };

    let insert = client
        .post("https://cnvmp3.com/insert_to_database.php")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&pid)
        .send()
        .await?
        .text()
        .await?;

    println!("{}", insert);

    Ok(())
}

async fn srv_download(
    client: reqwest::Client,
    youtube_url: String,
    title: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let pdv = PayloadDownloadVideo {
        format_value: 1,
        url: youtube_url,
        quality: 5,
        title: title,
    };

    let download_video = client
        .post("https://cnvmp3.com/download_video.php")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&pdv)
        .send()
        .await?
        .text()
        .await?;

    let dv_response: ResponseDownloadVideo = match json_parse(&download_video) {
        Ok(parsed) => parsed,
        Err(error) => panic!("Error parsing json: {error}"),
    };

    let server_path = dv_response.download_link;

    Ok(server_path)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // collect program args
    let mut argv: Args = args();

    // for now just grab the first arg (not the program name, obv)
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

    /*
     * https://www.youtube.com/watch?v=sidhfs978
     * <scheme>://<host>/<path>/<query_pairs>
     */

    let youtube_id = match youtube_url.query_pairs().next() {
        Some(v) => v.1,
        None => panic!("bye"),
    };

    let pcd = PayloadCheckDatabase {
        youtube_id: youtube_id.to_string(),
        quality: 5,
        format_value: 1,
    };

    let client = reqwest::Client::new();

    let checkdb_res_text = client
        .post("https://cnvmp3.com/check_database.php")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&pcd)
        .send()
        .await?
        .text()
        .await?;

    let checkdb_res_value: Value = match serde_json::from_str(&checkdb_res_text) {
        Ok(data) => data,
        Err(error) => panic!("{:?}", error),
    };

    match match_response(checkdb_res_value.clone()) {
        ResponseCheckDatabase::Exist(data) => {
            eprintln!("[INFO] found file in cdn-local storage");
            // 1. check if already exists locally
            if Path::new(format!("mp3/{}.mp3", data.data.title).as_str()).exists() {
                println!("video has already been locally saved as mp3");
                return Ok(());
            }

            cdn_download(
                client,
                data.data.server_path.clone(),
                data.data.title.clone(),
            )
            .await?;
        }
        ResponseCheckDatabase::NoExist(_) => {
            eprintln!("[INFO] unable to find file in cdn-local storage");
            let title = cdn_fetch(client.clone(), youtube_url.as_str()).await?;

            let server_path =
                srv_download(client.clone(), youtube_url.to_string(), title.clone()).await?;

            cdn_insert(
                client.clone(),
                server_path.clone(),
                title.clone(),
                youtube_id.to_string(),
            )
            .await?;

            cdn_download(client, server_path, title).await?;
        }
        ResponseCheckDatabase::Unknown(unknown) => {
            eprintln!("unknown response type: {:?}", unknown);
            exit(1);
        }
    };

    Ok(())
}
