#[derive(Debug)]
pub enum HttpVersion {
    // TODO support 0.9 and 1.0
    Version1_1,
    Version2_0,
}

impl ToString for HttpVersion {
    fn to_string(&self) -> String {
        match self {
            HttpVersion::Version1_1 => "HTTP/1.1".to_string(),
            HttpVersion::Version2_0 => "HTTP/2".to_string(),
        }
    }
}

impl TryFrom<&str> for HttpVersion {
    // TODO error
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "HTTP/1.1" => Ok(HttpVersion::Version1_1),
            "HTTP/2" => Ok(HttpVersion::Version2_0),
            _ => Err(()),
        }
    }
}