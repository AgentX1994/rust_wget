use std::collections::HashMap;

use super::HttpVersion;

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch
}

impl ToString for HttpMethod {
    fn to_string(&self) -> String {
        match self {
            HttpMethod::Get => "GET".to_string(),
            HttpMethod::Head => "HEAD".to_string(),
            HttpMethod::Post => "POST".to_string(),
            HttpMethod::Put => "PUT".to_string(),
            HttpMethod::Delete => "DELETE".to_string(),
            HttpMethod::Connect => "CONNECT".to_string(),
            HttpMethod::Options => "OPTIONS".to_string(),
            HttpMethod::Trace => "TRACE".to_string(),
            HttpMethod::Patch => "PATCH".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    method: HttpMethod,
    path: String,
    version: HttpVersion,
    headers: HashMap<String, String>,
}

impl HttpRequest {
    pub fn new(method: HttpMethod, path: String, version: HttpVersion) -> Self {
        HttpRequest {
            method,
            path,
            version,
            headers: HashMap::new()
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = format!("{} {} {}\r\n", self.method.to_string(), self.path, self.version.to_string());
        for (key, value) in &self.headers {
            data += key;
            data += ": ";
            data += value;
            data += "\r\n"
        }
        data += "\r\n\r\n";
        data.into_bytes()
    }

    pub fn add_header<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.headers.insert(key.into(), value.into());
    }
}