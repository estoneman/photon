use regex::Regex;
use url::Url;

use crate::error::{Error, ErrorKind};

const PATTERN_EMBED: &str = r"^\/embed";
const PATTERN_SHORT: &str = r"^\/shorts";
const PATTERN_REGULAR: &str = r"^\/watch";

#[derive(Clone, Debug)]
pub enum YouTubeURLKind {
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
pub struct YouTubeURL {
    pub url: Url,
    pub r#type: YouTubeURLKind,
    pub id: String,
}

impl YouTubeURL {
    pub fn new(url: Url) -> Result<Self, Error> {
        let r#type = YouTubeURL::get_type(url.clone())?;
        let id = YouTubeURL::get_id(url.clone(), r#type.clone())?;

        let youtube_url = YouTubeURL { url, r#type, id };

        if let Err(e) = youtube_url.validate() {
            return Err(Error {
                kind: ErrorKind::InvalidURL,
                value: format!("error: {e:?}"),
            });
        };

        Ok(youtube_url)
    }

    pub fn get_type(url: Url) -> Result<YouTubeURLKind, Error> {
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

    pub fn validate(&self) -> Result<(), Error> {
        let pattern = Regex::new(
            r"(?:youtube\.com\/(?:[^\/]+\/.+\/|(?:v|embed|watch|shorts)\/|.*[?&]v=)|youtu\.be\/)([a-zA-Z0-9_-]{11})(?:[&?]|$)"
        ).unwrap();

        if !pattern.is_match(self.url.as_str()) {
            return Err(Error {
                kind: ErrorKind::InvalidURL,
                value: format!("bad url: {}", self.url.as_str()),
            });
        }

        if let YouTubeURLKind::Invalid = self.r#type {
            return Err(Error {
                kind: ErrorKind::InvalidURLType,
                value: format!("bad type: {}", self.r#type),
            });
        };

        Ok(())
    }

    pub fn get_id(url: Url, r#type: YouTubeURLKind) -> Result<String, Error> {
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

mod tests {
    #[allow(unused_imports)]
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
}
