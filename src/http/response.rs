use std::{collections::HashMap, str::FromStr, fmt};

use super::HttpVersion;

#[derive(Debug)]
pub enum HttpStatus {
    Ok,
    MovedPermanently
}

impl FromStr for HttpStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let status_code = s.parse::<u16>().or(Err(()))?;
        
        match status_code {
            200 => Ok(HttpStatus::Ok),
            301 => Ok(HttpStatus::MovedPermanently),
            _ => unimplemented!()
        }
    }
}


impl fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            HttpStatus::Ok => 200u16,
            HttpStatus::MovedPermanently => 301u16,
        };
        write!(f, "{}", code)
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    version: HttpVersion,
    status_code: HttpStatus,
    status_message: String,
    headers: HashMap<String, String>,
    data: Vec<u8>
}

impl HttpResponse {
    pub fn new(version: HttpVersion, status_code: HttpStatus, status_message: String) -> Self {
        HttpResponse {
            version,
            status_code,
            status_message,
            headers: HashMap::new(),
            data: Vec::new(),
        }
    }

    pub fn add_header<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.headers.insert(key.into(), value.into());
    }

    pub fn set_data<D: Into<Vec<u8>>>(&mut self, data: D) {
        self.data = data.into();
    }

    pub fn get_header(&self, k: &str) -> Option<&str> {
        self.headers.get(k).map(
            |s| &**s
        )
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}\r\n", self.version.to_string(), self.status_code, self.status_message)?;
        for (key, value) in &self.headers {
            write!(f, "{}: {}\r\n", key, value)?;
        }
        write!(f, "{}\r\n\r\n", String::from_utf8_lossy(&self.data))
    }
}