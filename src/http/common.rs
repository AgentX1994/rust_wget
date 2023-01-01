use core::fmt;

#[derive(Debug)]
pub enum HttpVersion {
    Version0_9,
    Version1_0,
    Version1_1,
    Version2_0,
}

impl TryFrom<&str> for HttpVersion {
    // TODO error
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "HTTP/0.9" => Ok(HttpVersion::Version0_9),
            "HTTP/1.0" => Ok(HttpVersion::Version1_0),
            "HTTP/1.1" => Ok(HttpVersion::Version1_1),
            "HTTP/2" => Ok(HttpVersion::Version2_0),
            _ => Err(()),
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
