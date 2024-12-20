use infer::audio::is_mp3;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fs::File;
use std::io::Write;

// check_database.php
#[derive(Debug, Serialize)]
struct PayloadCheckDatabase {
    #[serde(rename = "formatValue")]
    format_value: i64,
    youtube_id: String,
    quality: i64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ResponseCheckDatabaseData {
    id: i64,
    quality: String,
    pub server_path: String,
    pub title: String,
    youtube_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ResponseCheckDatabaseExist {
    pub data: ResponseCheckDatabaseData,
    success: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ResponseCheckDatabaseNoExist {
    error: String,
    success: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseCheckDatabase {
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
    url: String,
    quality: i64,
    title: String,
    #[serde(rename = "formatValue")]
    format_value: i64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ResponseDownloadVideo {
    download_link: String,
    success: bool,
}

// insert_database.php
#[derive(Debug, Serialize)]
struct PayloadInsertDatabase {
    youtube_id: String,
    server_path: String,
    quality: i64,
    title: String,
    #[serde(rename = "formatValue")]
    format_value: i64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ResponseInsertDatabase {
    success: bool,
    message: String,
}

pub struct CNVClient {
    pub client: reqwest::Client,
}

pub trait CNVRequester {
    async fn cdn_download(
        &self,
        server_path: String,
        title: String,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn cdn_fetch(&self, url: &str) -> Result<String, Box<dyn std::error::Error>>;

    async fn cdn_insert(
        &self,
        server_path: String,
        title: String,
        youtube_id: String,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn srv_download(
        &self,
        youtube_url: String,
        title: String,
    ) -> Result<String, Box<dyn std::error::Error>>;

    async fn check_database(&self, youtube_id: String)
        -> Result<Value, Box<dyn std::error::Error>>;
}

impl CNVRequester for CNVClient {
    async fn cdn_download(
        &self,
        server_path: String,
        title: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let download = self.client.get(server_path).send().await?.bytes().await?;

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

    async fn cdn_fetch(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let pgvd = PayloadGetVideoData {
            url: url.to_string(),
        };

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

    async fn cdn_insert(
        &self,
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

        let insert = self
            .client
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
        &self,
        youtube_url: String,
        title: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let pdv = PayloadDownloadVideo {
            format_value: 1,
            url: youtube_url,
            quality: 5,
            title: title,
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

    async fn check_database(
        &self,
        youtube_id: String,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let pcd = PayloadCheckDatabase {
            youtube_id: youtube_id,
            quality: 5,
            format_value: 1,
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
}

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

pub fn json_parse<T>(raw: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    serde_json::from_str::<T>(raw).map_err(|e| e.to_string())
}
