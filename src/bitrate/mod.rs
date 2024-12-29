use serde::{Deserialize, Serialize};

/// Enumerated list of supported levels of bit rates:
/// * `Kbps320` => 320 kb/s
/// * `Kbps256` => 256 kb/s
/// * `Kbps128` => 128 kb/s
/// * `Kbps96`  => 96 kb/s
///
/// *NOTE*: value of each variant is assigned as seen in cnvmp3.com source code
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(usize)]
pub enum BitRate {
    Kbps320 = 0,
    Kbps256 = 1,
    Kbps128 = 4, // default
    Kbps96 = 5,
}

#[derive(Debug)]
pub struct FromNumberError<T> {
    value: T,
}

impl<T: std::fmt::Display> std::fmt::Display for FromNumberError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Invalid number: {}", self.value)
    }
}

pub trait FromNumber<T>: Sized {
    fn from_number(n: T) -> Result<Self, FromNumberError<T>>;
}

impl<T: Into<u64> + Copy> FromNumber<T> for BitRate {
    fn from_number(n: T) -> Result<Self, FromNumberError<T>> {
        match n.into() {
            320 => Ok(BitRate::Kbps320),
            256 => Ok(BitRate::Kbps256),
            128 => Ok(BitRate::Kbps128),
            96 => Ok(BitRate::Kbps96),
            _ => Err(FromNumberError { value: n }),
        }
    }
}
