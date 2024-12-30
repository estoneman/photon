#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub value: String,
}

#[derive(Debug)]
pub enum ErrorKind {
    InvalidURL,
    InvalidURLType,
    InvalidJSON,
    CNVResponseError,
    ReqwestError,
    SerdeError,
    BoxError,
    Error,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::InvalidURL => writeln!(f, "InvalidURL"),
            Self::InvalidURLType => writeln!(f, "InvalidURLType"),
            Self::InvalidJSON => writeln!(f, "InvalidJSON"),
            Self::CNVResponseError => writeln!(f, "JSONParseError"),
            Self::ReqwestError => writeln!(f, "ReqwestError"),
            Self::SerdeError => writeln!(f, "SerdeError"),
            Self::BoxError => writeln!(f, "BoxError"),
            Self::Error => writeln!(f, "Error"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Kind: {}, Message: {}", self.kind, self.value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error {
            kind: ErrorKind::ReqwestError,
            value: format!("error: reqwest ({})", value),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error {
            kind: ErrorKind::SerdeError,
            value: format!("error: serde ({})", value),
        }
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Error {
            kind: ErrorKind::BoxError,
            value: format!("error: box ({})", value),
        }
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error {
            kind: ErrorKind::Error,
            value: value.to_string(),
        }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error {
            kind: ErrorKind::Error,
            value,
        }
    }
}

impl std::error::Error for Error {}
