use std::str::FromStr;

use crate::error::{WgetError, WgetResult};

#[derive(Debug, PartialEq, Eq)]
pub enum Protocol {
    // TODO allow https?
    Http,
    Https,
    Ftp,
}

impl Protocol {
    pub fn get_port(&self) -> u16 {
        match self {
            Protocol::Http => 80,
            Protocol::Https => 443,
            Protocol::Ftp => 21,
        }
    }
}

impl FromStr for Protocol {
    type Err = WgetError;

    fn from_str(s: &str) -> WgetResult<Self> {
        match s {
            "http:" => Ok(Protocol::Http),
            "https:" => Ok(Protocol::Https),
            "ftp:" => Ok(Protocol::Ftp),
            _ => Err(WgetError::ParsingError(format!("Unknown protocol: {}", s))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_default_port_for_protocol() {
        assert_eq!(Protocol::Http.get_port(), 80);
        assert_eq!(Protocol::Https.get_port(), 443);
        assert_eq!(Protocol::Ftp.get_port(), 21);
    }

    #[test]
    fn parses_protocol_from_str() {
        assert!(matches!("http:".parse(), Ok(Protocol::Http)));
        assert!(matches!("https:".parse(), Ok(Protocol::Https)));
        assert!(matches!("ftp:".parse(), Ok(Protocol::Ftp)));
        assert!(matches!(
            "not a protocol".parse::<Protocol>(),
            Err(WgetError::ParsingError(_))
        ));
    }
}
