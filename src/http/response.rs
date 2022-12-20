use std::{collections::HashMap, str::FromStr};

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
}