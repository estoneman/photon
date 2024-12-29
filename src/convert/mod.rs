use infer::audio::is_mp3;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use url::Url;

use crate::bitrate::BitRate;

const PATTERN_EMBED: &str = r"^\/embed";
const PATTERN_SHORT: &str = r"^\/shorts";
const PATTERN_REGULAR: &str = r"^\/watch";

/// Enumerated list of supported formats to download youtube videos as
/// * MP3 for audio
/// * MP4 for video
#[derive(Debug, Deserialize, Serialize)]
#[repr(usize)]
enum DLFormat {
    MP4 = 0,
    MP3 = 1,
}

/// Payload to send to `check_database.php` endpoint
/// Used to retrieve video metadata as described by `CheckDatabaseVideoData`
#[derive(Debug, Serialize)]
struct PayloadCheckDatabase {
    #[serde(rename = "formatValue")]
    format_value: usize,
    quality: BitRate,
    youtube_id: String,
}

/// When a video is found in its database, cnvmp3 will return this video data
#[derive(Debug, Deserialize)]
struct CheckDatabaseVideoData {
    #[serde(rename = "id")]
    _id: i64,
    #[serde(rename = "quality")]
    _quality: String, // NOTE: this is a String in the response, but number in the payload
    server_path: String,
    #[serde(rename = "title")]
    _title: String,
    #[serde(rename = "youtube_id")]
    _youtube_id: String,
}

/// When a video is found in the cnvmp3 database
#[derive(Debug, Deserialize)]
struct CheckDatabaseExist {
    data: CheckDatabaseVideoData,
    #[serde(rename = "success")]
    _success: bool,
}

/// When a video is not found in the cnvmp3 database
/// `error` will describe what happened on cnvmp3's side
#[derive(Debug, Deserialize)]
struct CheckDatabaseNoExist {
    error: String,
    #[serde(rename = "success")]
    _success: bool,
}

/// Payload to send to `get_video_data.php` endpoint
/// Used to retrieve the title of the YouTube video
#[derive(Debug, Serialize)]
struct PayloadGetVideoData {
    url: Url,
}

/// When successful, cnvmp3 will return the title of the video
#[derive(Debug, Deserialize)]
struct GetVideoData {
    #[serde(rename = "success")]
    _success: bool,
    title: String,
}

/// When a failure occurs, cnvmp3 will return the error encountered
#[derive(Debug, Deserialize)]
struct GetVideoDataError {
    #[serde(rename = "success")]
    _success: bool,
    error: String,
}

/// Payload to send to `download_video.php` endpoint
/// Used to retrieve the remote location in cnvmp3's cdn where the MP3 file
/// is hosted
#[derive(Debug, Serialize)]
struct PayloadDownloadVideo {
    #[serde(rename = "formatValue")]
    format_value: usize,
    quality: BitRate,
    title: String,
    url: Url,
}

/// When successful, cnvmp3 will return the remote location from which the MP3 file can be download
#[derive(Debug, Deserialize)]
struct DownloadVideoData {
    download_link: String,
    #[serde(rename = "success")]
    _success: bool,
}

/// When the MP3 file could not be downloaded into one of the hosts in the cdn
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct DownloadVideoError {
    error: String,
    #[serde(rename = "errorType")]
    error_type: i64,
    success: bool,
}

/// Payload to send to `insert_to_database.php` endpoint
/// Used as an entry into the cnvmp3 database
#[derive(Debug, Serialize)]
struct PayloadInsertToDatabase {
    #[serde(rename = "formatValue")]
    format_value: usize,
    quality: BitRate,
    server_path: String,
    title: String,
    youtube_id: String,
}

/// Upon success, cnvmp3 will return the success messsage
#[derive(Debug, Deserialize)]
struct InsertToDatabaseData {
    #[serde(rename = "success")]
    _success: bool,
    message: String,
}

/// Upon failure, the error encountered will be returned
#[derive(Debug, Deserialize)]
struct InsertToDatabaseError {
    #[serde(rename = "success")]
    _success: bool,
    error: String,
}

/// Custom wrapper for `reqwest::Client`
#[allow(dead_code)]
struct CNVClient {
    client: reqwest::Client,
    dest_type: String,
}

/// Implementation of the responsibilities of my custom client
impl CNVClient {
    /// Sends a payload to the `/check_database.php` endpoint to determine whether
    /// the metadata for an MP3 file is available. If found, the metadata includes
    /// the remote location for downloading via the custom client (`cdn_download`).
    ///
    /// # Arguments
    ///
    /// * `youtube_id` - A `String` representing the unique identifier of the YouTube video.
    ///                  This ID is used to query the database for metadata associated
    ///                  with the corresponding MP3 file.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `bool`:
    ///
    /// - `true` if the metadata is found and successfully parsed.
    /// - `false` if the metadata is not found or the `success` field in the response is absent.
    ///
    /// If an error occurs during the HTTP request or JSON deserialization, it returns an error
    /// wrapped in a `Box<dyn std::error::Error>`.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - The HTTP request to the server fails.
    /// - The response cannot be deserialized as valid JSON.
    /// - The `success` field is missing or invalid in the JSON response.
    async fn check_database(
        &self,
        youtube_id: String,
        quality: BitRate,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let format_value = DLFormat::MP3 as usize;

        let pcd = PayloadCheckDatabase {
            format_value,
            quality,
            youtube_id,
        };

        let checkdb_res = self
            .client
            .post("https://cnvmp3.com/check_database.php")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&pcd)
            .send()
            .await?
            .bytes()
            .await?;

        let checkdb_parsed: Value = serde_json::from_slice(checkdb_res.as_ref())?;

        Ok(checkdb_parsed)
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
    async fn cdn_fetch(&self, url: Url) -> Result<Value, Box<dyn std::error::Error>> {
        let pgvd = PayloadGetVideoData { url };

        let gvd_res = self
            .client
            .post("https://cnvmp3.com/get_video_data.php")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&pgvd)
            .send()
            .await?
            .bytes()
            .await?;

        let gvd_parsed: Value = serde_json::from_slice(gvd_res.as_ref())?;

        Ok(gvd_parsed)
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
    async fn srv_download(
        &self,
        url: Url,
        title: String,
        quality: BitRate,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let format_value = DLFormat::MP3 as usize;

        let pdv = PayloadDownloadVideo {
            format_value,
            quality,
            title,
            url,
        };

        let dv_res = self
            .client
            .post("https://cnvmp3.com/download_video.php")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&pdv)
            .send()
            .await?
            .bytes()
            .await?;

        let dv_parsed: Value = serde_json::from_slice(dv_res.as_ref())?;

        Ok(dv_parsed)
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
    async fn cdn_insert(
        &self,
        server_path: String,
        title: String,
        youtube_id: String,
        quality: BitRate,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let format_value = DLFormat::MP3 as usize;

        let pid = PayloadInsertToDatabase {
            format_value,
            quality,
            server_path,
            title,
            youtube_id,
        };

        let ins_res = self
            .client
            .post("https://cnvmp3.com/insert_to_database.php")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&pid)
            .send()
            .await?
            .bytes()
            .await?;

        let ins_parsed: Value = serde_json::from_slice(ins_res.as_ref())?;

        Ok(ins_parsed)
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
    async fn cdn_download(
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

            if let Err(e) = outfile.write_all(&download) {
                return Err(format!("{:?}", e).into());
            }
        } else {
            return Err("downloaded content is not an mp3 file".into());
        }

        Ok(())
    }
}

#[derive(Debug)]
struct PhotonError {
    kind: PhotonErrorKind,
    msg: String,
}

#[derive(Debug)]
enum PhotonErrorKind {
    InvalidURL,
    InvalidURLType,
}

impl std::fmt::Display for PhotonErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::InvalidURL => writeln!(f, "InvalidURL"),
            Self::InvalidURLType => writeln!(f, "InvalidURLType"),
        }
    }
}

impl std::fmt::Display for PhotonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Kind: {}, Message: {}", self.kind, self.msg)
    }
}

impl std::error::Error for PhotonError {}

#[derive(Clone, Debug)]
enum YouTubeURLKind {
    Short,
    Embed,
    Regular,
    Invalid,
}

impl std::fmt::Display for YouTubeURLKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YouTubeURLKind::Short => writeln!(f, "Short"),
            YouTubeURLKind::Embed => writeln!(f, "Embed"),
            YouTubeURLKind::Regular => writeln!(f, "Regular"),
            YouTubeURLKind::Invalid => writeln!(f, "Invalid"),
        }
    }
}

#[derive(Clone, Debug)]
struct YouTubeURL {
    url: Url,
    r#type: YouTubeURLKind,
    id: String,
}

impl YouTubeURL {
    fn new(url: Url) -> Result<Self, PhotonError> {
        let r#type = YouTubeURL::get_type(url.clone())?;
        let id = YouTubeURL::get_id(url.clone(), r#type.clone())?;

        let youtube_url = YouTubeURL { url, r#type, id };

        if let Err(e) = youtube_url.validate() {
            return Err(PhotonError {
                kind: PhotonErrorKind::InvalidURL,
                msg: format!("error: {e:?}"),
            });
        };

        Ok(youtube_url)
    }

    fn get_type(url: Url) -> Result<YouTubeURLKind, PhotonError> {
        let embed_pattern = Regex::new(PATTERN_EMBED).unwrap();
        let short_pattern = Regex::new(PATTERN_SHORT).unwrap();
        let regular_pattern = Regex::new(PATTERN_REGULAR).unwrap();

        let path = url.path();

        let r#type = if regular_pattern.is_match(path) {
            YouTubeURLKind::Regular
        } else if short_pattern.is_match(path) {
            YouTubeURLKind::Short
        } else if embed_pattern.is_match(path) {
            YouTubeURLKind::Embed
        } else {
            YouTubeURLKind::Invalid
        };

        Ok(r#type)
    }

    fn validate(&self) -> Result<(), PhotonError> {
        let pattern = Regex::new(
            r"(?:youtube\.com\/(?:[^\/]+\/.+\/|(?:v|embed|watch|shorts)\/|.*[?&]v=)|youtu\.be\/)([a-zA-Z0-9_-]{11})(?:[&?]|$)"
        ).unwrap();

        if !pattern.is_match(self.url.as_str()) {
            return Err(PhotonError {
                kind: PhotonErrorKind::InvalidURL,
                msg: format!("bad url: {}", self.url.as_str()),
            });
        }

        if let YouTubeURLKind::Invalid = self.r#type {
            return Err(PhotonError {
                kind: PhotonErrorKind::InvalidURLType,
                msg: format!("bad type: {}", self.r#type),
            });
        };

        Ok(())
    }

    fn get_id(url: Url, r#type: YouTubeURLKind) -> Result<String, PhotonError> {
        let mut youtube_id = String::from("");

        match r#type {
            YouTubeURLKind::Invalid => {
                youtube_id = String::from("invalid");
            }
            _ => {
                let id_pattern =
                    Regex::new(r"(?:(?:shorts|embed)\/(\S+)\/?)|(?:watch\?v=(\S+))").unwrap();

                for (_, [id]) in id_pattern.captures_iter(url.as_str()).map(|c| c.extract()) {
                    youtube_id = String::from(id);
                }
            }
        }

        Ok(youtube_id)
    }
}

/// Converts a YouTube video to an MP3 file and downloads it.
///
/// # Arguments
///
/// * `youtube_url` - The URL of the YouTube video to convert.
/// * `dest_type` - The destination type for the MP3 file download.
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
///     let dest_type = String::from("local");
///     if let Err(e) = download(youtube_url, dest_type).await {
///         eprintln!("Error: {}", e);
///     } else {
///         println!("Download successful!");
///     }
/// }
/// ```
///
/// # Notes
///
/// This function uses a custom CDN service to perform the conversion and
/// downloading process. It handles checking whether the video is already
/// saved as an MP3, fetching video data, inserting into the database, and
/// downloading the MP3 file.
#[tokio::main]
pub async fn download(
    url: Url,
    dest_type: String,
    quality: BitRate,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("info: using bitrate = {quality:?}");

    let youtube_url = YouTubeURL::new(url).unwrap();

    if Path::new(format!("mp3/{}.mp3", youtube_url.id).as_str()).exists() {
        println!("info: the requested video has already been saved locally as mp3");
        return Ok(());
    }

    let client = reqwest::Client::new();

    let c = CNVClient { client, dest_type };

    let checkdb_res = c.check_database(youtube_url.id.clone(), quality).await?;

    let success = match checkdb_res.get("success").and_then(|v| v.as_bool()) {
        Some(s) => s,
        None => {
            return Err(
                "Unable to reference field `success` in /check_database.php response".into(),
            )
        }
    };

    match success {
        // response contains server_path, use it
        true => {
            let checkdb: CheckDatabaseExist =
                serde_json::from_value::<CheckDatabaseExist>(checkdb_res)
                    .expect("Parsing as CheckDatabaseExist should work");
            c.cdn_download(checkdb.data.server_path, youtube_url.id)
                .await?;
        }
        // response does not contain server_path, go get it
        false => {
            let error: CheckDatabaseNoExist =
                serde_json::from_value::<CheckDatabaseNoExist>(checkdb_res)
                    .expect("Parsing as CheckDatabaseNoExist should work");
            eprintln!("info: {}", error.error);

            let getvd_res = c.cdn_fetch(youtube_url.url.clone()).await?;

            let success = match getvd_res.get("success").and_then(|v| v.as_bool()) {
                Some(s) => s,
                None => {
                    return Err(
                        "Unable to reference field `success` in /get_video_data.php response"
                            .into(),
                    )
                }
            };

            if !success {
                let error: GetVideoDataError =
                    serde_json::from_value::<GetVideoDataError>(getvd_res)
                        .expect("Parsing as GetVideoDataError should work");
                return Err(format!("/get_video_data.php failed.. {}", error.error).into());
            }

            let getvd: GetVideoData = serde_json::from_value::<GetVideoData>(getvd_res)
                .expect("Parsing as GetVideoData should work");

            let title = getvd.title;

            let dv_res = c
                .srv_download(youtube_url.url, title.clone(), quality)
                .await?;

            let success = match dv_res.get("success").and_then(|v| v.as_bool()) {
                Some(s) => s,
                None => {
                    return Err(
                        "Unable to reference field `success` in /download_video.php response"
                            .into(),
                    )
                }
            };

            if !success {
                let error: DownloadVideoError =
                    serde_json::from_value::<DownloadVideoError>(dv_res)
                        .expect("Parsing as DownloadVideoError should work");
                return Err(format!("/download_video.php failed.. {}", error.error).into());
            }

            let dv: DownloadVideoData = serde_json::from_value::<DownloadVideoData>(dv_res)
                .expect("Parsing as DownloadVideoData should work");

            let download_link = dv.download_link;

            let dl_res = c
                .cdn_insert(
                    download_link.clone(),
                    title,
                    youtube_url.id.clone(),
                    quality,
                )
                .await?;

            let success =
                match dl_res.get("success").and_then(|v| v.as_bool()) {
                    Some(s) => s,
                    None => return Err(
                        "Unable to reference field `success` in /insert_to_database.php response"
                            .into(),
                    ),
                };

            if !success {
                let error: InsertToDatabaseError =
                    serde_json::from_value::<InsertToDatabaseError>(dl_res)
                        .expect("Parsing as InsertToDatabaseError should work");
                return Err(format!("/insert_to_database.php failed.. {}", error.error).into());
            }

            let dl: InsertToDatabaseData = serde_json::from_value::<InsertToDatabaseData>(dl_res)
                .expect("Parsing as InsertToDatabaseData should work");

            eprintln!("info: {}", dl.message);

            if let Err(e) = c.cdn_download(download_link, youtube_url.id).await {
                return Err(format!("error: {}", e).into());
            };
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_id() {
        let test_cases = vec![
            (
                "https://www.youtube.com/watch?v=yPvoKz6tyJs",
                YouTubeURLKind::Regular,
                "yPvoKz6tyJs",
            ),
            (
                "https://www.youtube.com/embed/3rLN_-VNcfs",
                YouTubeURLKind::Embed,
                "3rLN_-VNcfs",
            ),
            (
                "https://www.youtube.com/shorts/3rLN_-VNcfs",
                YouTubeURLKind::Short,
                "3rLN_-VNcfs",
            ),
            (
                "https://www.youtube.com/invalid/invalid",
                YouTubeURLKind::Invalid,
                "invalid",
            ),
        ];

        for (url, r#type, exp) in test_cases {
            let id = YouTubeURL::get_id(Url::parse(url).unwrap(), r#type).unwrap();
            assert_eq!(id, exp);
        }
    }

    #[test]
    fn test_get_type() {
        let test_cases = vec![
            (
                "https://www.youtube.com/shorts/3rLN_-VNcfs",
                YouTubeURLKind::Short,
            ),
            (
                "https://www.youtube.com/embed/3rLN_-VNcfs",
                YouTubeURLKind::Embed,
            ),
            (
                "https://www.youtube.com/watch?v=yPvoKz6tyJs",
                YouTubeURLKind::Regular,
            ),
            (
                "https://www.youtube.com/invalid/invalid",
                YouTubeURLKind::Invalid,
            ),
        ];

        for (url, exp) in test_cases {
            let r#type = YouTubeURL::get_type(Url::parse(url).unwrap()).unwrap();
            assert_eq!(r#type.to_string(), exp.to_string());
        }
    }

    #[test]
    fn test_download() {
        let youtube_url = Url::parse("https://www.youtube.com/watch?v=yPvoKz6tyJs")
            .expect("Url::parse should work");
        let dest_type = String::from("local");

        let result = download(youtube_url.clone(), dest_type.clone(), BitRate::Kbps96);
        assert!(result.is_ok());
    }
}
