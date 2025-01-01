use serde::{Deserialize, Serialize};
use url::Url;

use crate::bitrate::BitRate;

/// Payload to send to `check_database.php` endpoint
/// Used to retrieve video metadata as described by `CheckDatabaseVideoData`
#[derive(Debug, Serialize)]
pub struct PayloadCheckDatabase {
    #[serde(rename = "formatValue")]
    pub format_value: usize,
    pub quality: BitRate,
    pub youtube_id: String,
}

/// Metadata of a YouTube video as defined by cnvmp3
#[derive(Debug, Deserialize)]
pub struct VideoData {
    #[serde(rename = "id")]
    _id: i64,
    #[serde(rename = "quality")]
    _quality: String, // NOTE: this is a String in the response, but number in the payload
    pub server_path: String,
    #[serde(rename = "title")]
    _title: String,
    #[serde(rename = "youtube_id")]
    _youtube_id: String,
}

/// Response schema of successfully fulfilled request to `/check_database.php`
#[derive(Debug, Deserialize)]
pub struct CheckDatabaseSuccess {
    #[serde(rename = "success")]
    pub _success: bool,
    pub data: VideoData,
}

/// Response schema of a failed request to `/check_database.php`
#[derive(Debug, Deserialize)]
pub struct CheckDatabaseFail {
    #[serde(rename = "success")]
    pub _success: bool,
    pub error: String,
}

/// When a video is found in the cnvmp3 database (Exist)
///
/// When a video is not found in the cnvmp3 database
/// `error` will describe what happened on cnvmp3's side
/// (NoExist)
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseCheckDatabase {
    Exist(CheckDatabaseSuccess),
    NoExist(CheckDatabaseFail),
}

/// Payload to send to `get_video_data.php` endpoint
/// Used to retrieve the title of the YouTube video
#[derive(Debug, Serialize)]
pub struct PayloadGetVideoData {
    pub url: Url,
}

/// Response schema upon successfully fulfilled request to `/get_video_data.php`
#[derive(Debug, Deserialize)]
pub struct GetVideoDataSuccess {
    #[serde(rename = "success")]
    pub _success: bool,
    pub title: String,
}

/// Response schema upon failed request to `/get_video_data.php`
#[derive(Debug, Deserialize)]
pub struct GetVideoDataFail {
    #[serde(rename = "success")]
    pub _success: bool,
    pub error: String,
}

/// Possibilities of responses to requests made to `/get_video_data.php`
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseGetVideoData {
    Success(GetVideoDataSuccess),
    Fail(GetVideoDataFail),
}

/// Payload to send to `download_video.php` endpoint
/// Used to retrieve the remote location in cnvmp3's cdn where the MP3 file
/// is hosted
#[derive(Debug, Serialize)]
pub struct PayloadDownloadVideo {
    #[serde(rename = "formatValue")]
    pub format_value: usize,
    pub quality: BitRate,
    pub title: String,
    pub url: Url,
}

/// Response schema upon successfully fulfilled request to `/download_video.php`
#[derive(Debug, Deserialize)]
pub struct DownloadVideoSuccess {
    pub download_link: String,
    #[serde(rename = "success")]
    pub _success: bool,
}

/// Response schema upon failed request to `/download_video.php`
#[derive(Debug, Deserialize)]
pub struct DownloadVideoFail {
    pub error: String,
    #[serde(rename = "errorType")]
    pub error_type: i64,
    pub _success: bool,
}

/// Possibilities of responses to requests made to `/download_video.php`
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseDownloadVideo {
    Success(DownloadVideoSuccess),
    Fail(DownloadVideoFail),
}

/// Payload to send to `insert_to_database.php` endpoint
/// Used as an entry into the cnvmp3 database
#[derive(Debug, Serialize)]
pub struct PayloadInsertToDatabase {
    #[serde(rename = "formatValue")]
    pub format_value: usize,
    pub quality: BitRate,
    pub server_path: String,
    pub title: String,
    pub youtube_id: String,
}

/// Response schema upon successfully fulfilled request to `/insert_to_database.php`
#[derive(Debug, Deserialize)]
pub struct InsertToDatabaseSuccess {
    #[serde(rename = "success")]
    pub _success: bool,
    pub message: String,
}

/// Response schema upon failed request to `/insert_to_database.php`
#[derive(Debug, Deserialize)]
pub struct InsertToDatabaseFail {
    #[serde(rename = "success")]
    pub _success: bool,
    pub error: String,
}

/// Possibilities of responses to requests made to `/insert_to_database.php`
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseInsertToDatabase {
    Success(InsertToDatabaseSuccess),
    Fail(InsertToDatabaseFail),
}
