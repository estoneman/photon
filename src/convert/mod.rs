use infer::audio::is_mp3;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use url::Url;

use crate::bitrate::BitRate;
use crate::error::{Error, ErrorKind};
use crate::youtube_url::YouTubeURL;

mod schema;
use schema::{
    CheckDatabaseFail, CheckDatabaseSuccess, DownloadVideoFail, DownloadVideoSuccess,
    GetVideoDataFail, GetVideoDataSuccess, InsertToDatabaseFail, InsertToDatabaseSuccess,
    PayloadCheckDatabase, PayloadDownloadVideo, PayloadGetVideoData, PayloadInsertToDatabase,
    ResponseCheckDatabase, ResponseDownloadVideo, ResponseGetVideoData, ResponseInsertToDatabase,
};

/// Enumerated list of supported formats to download youtube videos as
/// * MP3 for audio
/// * MP4 for video
#[derive(Debug, Deserialize, Serialize)]
#[repr(usize)]
enum DLFormat {
    MP4 = 0,
    MP3 = 1,
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
    ) -> Result<ResponseCheckDatabase, Error> {
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
            .await
            .map_err(|e| Error {
                kind: ErrorKind::ReqwestError,
                value: format!("HTTP request failed: {}", e),
            })?
            .bytes()
            .await
            .map_err(|e| Error {
                kind: ErrorKind::ReqwestError,
                value: format!("Failed to read response as bytes: {}", e),
            })?;

        let checkdb_parsed: ResponseCheckDatabase = serde_json::from_slice(checkdb_res.as_ref())?;

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
    async fn cdn_fetch(&self, url: Url) -> Result<ResponseGetVideoData, Error> {
        let pgvd = PayloadGetVideoData { url };

        let gvd_res = self
            .client
            .post("https://cnvmp3.com/get_video_data.php")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&pgvd)
            .send()
            .await
            .map_err(|e| Error {
                kind: ErrorKind::ReqwestError,
                value: format!("HTTP request failed: {}", e),
            })?
            .bytes()
            .await
            .map_err(|e| Error {
                kind: ErrorKind::ReqwestError,
                value: format!("Failed to read response as bytes: {}", e),
            })?;

        let gvd_parsed: ResponseGetVideoData = serde_json::from_slice(gvd_res.as_ref())?;

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
    ) -> Result<ResponseDownloadVideo, Error> {
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
            .await
            .map_err(|e| Error {
                kind: ErrorKind::ReqwestError,
                value: format!("HTTP request failed: {}", e),
            })?
            .bytes()
            .await
            .map_err(|e| Error {
                kind: ErrorKind::ReqwestError,
                value: format!("Failed to read response as bytes: {}", e),
            })?;

        let dv_parsed: ResponseDownloadVideo = serde_json::from_slice(dv_res.as_ref())?;

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
    ) -> Result<ResponseInsertToDatabase, Error> {
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
            .await
            .map_err(|e| Error {
                kind: ErrorKind::ReqwestError,
                value: format!("HTTP request failed: {}", e),
            })?
            .bytes()
            .await
            .map_err(|e| Error {
                kind: ErrorKind::ReqwestError,
                value: format!("Failed to read response as bytes: {}", e),
            })?;

        let ins_parsed: ResponseInsertToDatabase = serde_json::from_slice(ins_res.as_ref())?;

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
pub async fn y2mp3(url: Url, dest_type: String, quality: BitRate) -> Result<(), Error> {
    eprintln!("info: using bitrate = {quality:?}");

    let youtube_url = YouTubeURL::new(url).unwrap();

    if Path::new(format!("mp3/{}.mp3", youtube_url.id).as_str()).exists() {
        println!("info: the requested video has already been saved locally as mp3");
        return Ok(());
    }

    let client = reqwest::Client::new();

    let c = CNVClient { client, dest_type };

    let checkdb_res = c.check_database(youtube_url.id.clone(), quality).await?;

    match checkdb_res {
        ResponseCheckDatabase::Exist(CheckDatabaseSuccess { data, _success }) => {
            if let Err(e) = c.cdn_download(data.server_path, youtube_url.id).await {
                return Err(format!("error: {}", e).into());
            }
        }
        ResponseCheckDatabase::NoExist(CheckDatabaseFail { _success, error }) => {
            eprintln!("info: {}", error);
            let gvd_res = c.cdn_fetch(youtube_url.url.clone()).await?;

            let title = match gvd_res {
                ResponseGetVideoData::Success(GetVideoDataSuccess { title, _success }) => title,
                ResponseGetVideoData::Fail(GetVideoDataFail { error, _success }) => {
                    return Err(Error {
                        kind: ErrorKind::CNVResponseError,
                        value: format!("get_video_data.php failed: {}", error),
                    });
                }
            };

            let dv_res = c
                .srv_download(youtube_url.url, title.clone(), quality)
                .await?;

            let dl_link = match dv_res {
                ResponseDownloadVideo::Success(DownloadVideoSuccess {
                    download_link,
                    _success,
                }) => download_link,
                ResponseDownloadVideo::Fail(DownloadVideoFail {
                    error,
                    error_type,
                    _success,
                }) => {
                    return Err(Error {
                        kind: ErrorKind::CNVResponseError,
                        value: format!("download_video.php failed: {} {}", error_type, error),
                    });
                }
            };

            let dl_res = c
                .cdn_insert(dl_link.clone(), title, youtube_url.id.clone(), quality)
                .await?;

            match dl_res {
                ResponseInsertToDatabase::Success(InsertToDatabaseSuccess {
                    message,
                    _success,
                }) => {
                    eprintln!("info: {}", message);
                }
                ResponseInsertToDatabase::Fail(InsertToDatabaseFail { error, _success }) => {
                    return Err(Error {
                        kind: ErrorKind::CNVResponseError,
                        value: format!("insert_to_database.php failed: {}", error),
                    });
                }
            }

            if let Err(e) = c.cdn_download(dl_link, youtube_url.id).await {
                return Err(format!("error: {}", e).into());
            }
        }
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_y2mp3() {
        let youtube_url = Url::parse("https://www.youtube.com/watch?v=yPvoKz6tyJs")
            .expect("Url::parse should work");
        let dest_type = String::from("local");

        let result = y2mp3(youtube_url.clone(), dest_type.clone(), BitRate::Kbps96);
        assert!(result.is_ok());
    }
}
