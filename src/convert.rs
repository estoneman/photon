use infer::audio::is_mp3;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use url::Url;

/// Payload to send to `check_database.php` endpoint
#[derive(Debug, Serialize)]
struct PayloadCheckDatabase {
    #[serde(rename = "formatValue")]
    format_value: i64,
    quality: i64,
    youtube_id: String,
}

/// When a video is found in its database, cnvmp3 will return its video data
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ResponseCheckDatabaseData {
    id: i64,
    quality: String,
    pub server_path: String,
    pub title: String,
    youtube_id: String,
}

/// When a video is found in the cnvmp3 database
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ResponseCheckDatabaseExist {
    pub data: ResponseCheckDatabaseData,
    success: bool,
}

/// When a video is not found in the cnvmp3 database
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ResponseCheckDatabaseNoExist {
    error: String,
    success: bool,
}

/// For determining if a video exists in the cnvmp3 database
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseCheckDatabase {
    Exist(ResponseCheckDatabaseExist),
    NoExist(ResponseCheckDatabaseNoExist),
    Unknown(Value),
}

/// Payload to send to `get_video_data.php` endpoint
#[derive(Debug, Serialize)]
struct PayloadGetVideoData {
    url: String,
}

/// When successful, cnvmp3 will return the title of the video
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ResponseGetVideoData {
    success: bool,
    title: String,
}

/// Payload to send to `download_video.php` endpoint
#[derive(Debug, Serialize)]
struct PayloadDownloadVideo {
    #[serde(rename = "formatValue")]
    format_value: i64,
    quality: i64,
    title: String,
    url: String,
}

/// When successful, cnvmp3 will return the remote location from which the MP3 file can be download
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ResponseDownloadVideo {
    download_link: String,
    success: bool,
}

/// Payload to send to `insert_to_database.php` endpoint
#[derive(Debug, Serialize)]
struct PayloadInsertToDatabase {
    #[serde(rename = "formatValue")]
    format_value: i64,
    quality: i64,
    server_path: String,
    title: String,
    youtube_id: String,
}

/// Regardless of success, cnvmp3 will return the status messsage
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ResponseInsertToDatabase {
    success: bool,
    message: String,
}

/// Custom wrapper for `reqwest::Client`
pub struct CNVClient {
    pub client: reqwest::Client,
}

/// Implementation of the responsibilities of my custom client
impl CNVClient {
    /// Sends a payload to the /check_database.php endpoint to determine if the
    /// MP3 file metadata is available. If found, the metadata includes the
    /// remote location for downloading via the custom client (`cdn_download`).
    ///
    /// # Arguments
    ///
    /// * `youtube_id` - A `String` representing the unique identifier of the YouTube video. This ID
    ///                  is used to query the database for metadata associated with the corresponding
    ///                  MP3 file.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `Value` (from `serde_json`) if the metadata is found, or an
    /// error (`Box<dyn std::error::Error>`) if the operation fails.
    pub async fn check_database(
        &self,
        youtube_id: String,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let format_value: i64 = 1;
        let quality: i64 = 5;

        let pcd = PayloadCheckDatabase {
            format_value,
            quality,
            youtube_id,
        };

        let checkdb_res_text = self
            .client
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

        Ok(checkdb_res_value)
    }

    /// Sends a request to `cnvmp3` to retrieve the YouTube video ID associated with the provided URL.
    ///
    /// # Arguments
    ///
    /// * `url` - A `String` containing the URL of the YouTube video. This URL is used to query the
    ///           `cnvmp3` service to obtain the corresponding YouTube video ID.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `String` with the YouTube video ID if the operation succeeds,
    /// or an error (`Box<dyn std::error::Error>`) if the request fails or the service does not return
    /// the expected response.
    pub async fn cdn_fetch(&self, url: String) -> Result<String, Box<dyn std::error::Error>> {
        let pgvd = PayloadGetVideoData { url };

        let gvd_res_text = self
            .client
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

    /// Sends a request for the cnvmp3 web server to find where the MP3 file is in the Content
    /// Delivery Network (CDN) for the given YouTube video.
    ///
    /// # Arguments
    ///
    /// * `url` - A `String` containing the URL of the YouTube video. This URL is used to identify
    ///           the video and locate the corresponding MP3 file in the CDN.
    /// * `title` - A `String` representing the title of the YouTube video. This may be used for
    ///             additional metadata or as part of the request to the `cnvmp3` web server.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `String` with the location of the MP3 file in the CDN if the
    /// operation is successful, or an error (`Box<dyn std::error::Error>`) if the request fails or
    /// the server does not return the expected response.
    pub async fn srv_download(
        &self,
        url: String,
        title: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let quality: i64 = 5;
        let format_value: i64 = 1;

        let pdv = PayloadDownloadVideo {
            format_value,
            quality,
            title,
            url,
        };

        let download_video = self
            .client
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

    /// Inserts video metadata into a local database to enable faster file retrieval in future requests.
    ///
    /// # Arguments
    ///
    /// * `server_path` - A `String` representing the path to the MP3 file on the server. This is used
    ///                   to locate the file when retrieving it from the local database.
    /// * `title` - A `String` containing the title of the YouTube video. This metadata is stored in
    ///             the local database for reference and identification purposes.
    /// * `youtube_id` - A `String` representing the unique identifier of the YouTube video. This ID
    ///                  is stored to associate the video metadata with the specific video.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with an empty tuple (`()`) on success, indicating that the metadata was
    /// successfully inserted into the database. On failure, returns an error (`Box<dyn std::error::Error>`).
    pub async fn cdn_insert(
        &self,
        server_path: String,
        title: String,
        youtube_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let format_value: i64 = 1;
        let quality: i64 = 5;

        let pid = PayloadInsertToDatabase {
            format_value,
            quality,
            server_path,
            title,
            youtube_id,
        };

        let ins_res_text = self
            .client
            .post("https://cnvmp3.com/insert_to_database.php")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&pid)
            .send()
            .await?
            .text()
            .await?;

        let ins_res_parsed: ResponseInsertToDatabase = match json_parse(&ins_res_text) {
            Ok(p) => p,
            Err(e) => panic!("Error parsing json: {e}\n{ins_res_text}"),
        };

        println!("info: {}", ins_res_parsed.message);

        Ok(())
    }

    /// Downloads the MP3 file from the specified remote location (`server_path`) and saves it locally.
    ///
    /// # Arguments
    ///
    /// * `server_path` - A `String` representing the remote path to the MP3 file on the server.
    ///                   This path is used to fetch the file for download.
    /// * `youtube_id` - A `String` containing the unique identifier of the YouTube video. This ID
    ///                  is used to associate the downloaded file with its source video.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with an empty tuple (`()`) on success, indicating the MP3 file was
    /// successfully downloaded and saved locally. On failure, returns an error (`Box<dyn std::error::Error>`).
    pub async fn cdn_download(
        &self,
        server_path: String,
        youtube_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let download = self
            .client
            .get(server_path)
            .header("Referer", "https://cnvmp3.com")
            .send()
            .await?
            .bytes()
            .await?;

        if is_mp3(&download) {
            let mut outfile = File::create(format!("mp3/{}.mp3", youtube_id))
                .expect("file creation should succeed");

            match outfile.write_all(&download) {
                Ok(_) => println!("info: {} saved successfully", youtube_id),
                Err(e) => println!("{:?}", e),
            }
        } else {
            println!("downloaded content is not an mp3 file");
        }

        Ok(())
    }
}

/// Checks the response from the `/check_database.php` endpoint to determine if it contains valid
/// data, an error message, or an unknown response.
///
/// # Arguments
///
/// * `value` - A `Value` (from `serde_json`) representing the response data received from the
///            `/check_database.php` endpoint.
///
/// # Returns
///
/// Returns a `ResponseCheckDatabase` enum indicating whether the response contains valid data,
/// an error message, or is unrecognized.
pub fn match_response(value: Value) -> ResponseCheckDatabase {
    if let Ok(data) = serde_json::from_value::<ResponseCheckDatabaseExist>(value.clone()) {
        let some_data: ResponseCheckDatabaseExist = data;
        return ResponseCheckDatabase::Exist(some_data);
    }

    if let Ok(error) = serde_json::from_value::<ResponseCheckDatabaseNoExist>(value.clone()) {
        let some_error: ResponseCheckDatabaseNoExist = error;
        return ResponseCheckDatabase::NoExist(some_error);
    }

    ResponseCheckDatabase::Unknown(value)
}

/// Attempts to parse a raw string of characters into the specified Rust type `T`.
///
/// # Arguments
///
/// * `raw` - A `&str` containing the raw JSON string to be parsed into a Rust type.
///
/// # Returns
///
/// Returns a `Result` containing the parsed value of type `T` on success, or an error message
/// as a `String` if the parsing fails.
///
/// # Type Parameters
///
/// * `T` - The target Rust type, which must implement `DeserializeOwned`.
pub fn json_parse<T>(raw: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    serde_json::from_str::<T>(raw).map_err(|e| e.to_string())
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
#[tokio::main]
pub async fn download(youtube_url: Url) -> Result<(), Box<dyn std::error::Error>> {
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
        println!("info: the requested video has already been saved locally as mp3");
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
