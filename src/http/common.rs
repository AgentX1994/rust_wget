use core::fmt;

use crate::error::{WgetError, WgetResult};

#[derive(Debug, PartialEq, Eq)]
pub enum HttpVersion {
    Version0_9,
    Version1_0,
    Version1_1,
    Version2_0,
}

impl TryFrom<&str> for HttpVersion {
    // TODO error
    type Error = WgetError;

    fn try_from(value: &str) -> WgetResult<Self> {
        match value {
            "HTTP/0.9" => Ok(HttpVersion::Version0_9),
            "HTTP/1.0" => Ok(HttpVersion::Version1_0),
            "HTTP/1.1" => Ok(HttpVersion::Version1_1),
            "HTTP/2" => Ok(HttpVersion::Version2_0),
            _ => Err(WgetError::ParsingError(format!(
                "Invalid Version {}",
                value
            ))),
        }
    }
}

impl fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version_str = match self {
            HttpVersion::Version0_9 => "HTTP/0.9".to_string(),
            HttpVersion::Version1_0 => "HTTP/1.0".to_string(),
            HttpVersion::Version1_1 => "HTTP/1.1".to_string(),
            HttpVersion::Version2_0 => "HTTP/2".to_string(),
        };
        write!(f, "{}", version_str)
    }
}

// TODO error handling
#[derive(Debug, PartialEq, Eq)]
pub enum Protocol {
    // TODO allow https?
    Http,
}

impl Protocol {
    pub fn get_port(&self) -> u16 {
        match self {
            Protocol::Http => 80,
        }
    }
}

#[derive(Debug, Default)]
pub struct Configuration {
    pub debug: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_http_version() {
        assert!(matches!("HTTP/0.9".try_into(), Ok(HttpVersion::Version0_9)));
        assert!(matches!("HTTP/1.0".try_into(), Ok(HttpVersion::Version1_0)));
        assert!(matches!("HTTP/1.1".try_into(), Ok(HttpVersion::Version1_1)));
        assert!(matches!("HTTP/2".try_into(), Ok(HttpVersion::Version2_0)));
        assert!(matches!(
            HttpVersion::try_from("not a version"),
            Err(WgetError::ParsingError(_))
        ));
    }

    #[test]
    fn version_to_string() {
        assert_eq!(HttpVersion::Version0_9.to_string(), "HTTP/0.9");
        assert_eq!(HttpVersion::Version1_0.to_string(), "HTTP/1.0");
        assert_eq!(HttpVersion::Version1_1.to_string(), "HTTP/1.1");
        assert_eq!(HttpVersion::Version2_0.to_string(), "HTTP/2");
    }

    #[test]
    fn gets_default_port_for_protocol() {
        assert_eq!(Protocol::Http.get_port(), 80);
    }
}
