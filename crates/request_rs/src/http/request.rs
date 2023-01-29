use std::fmt;

use unicase::UniCase;

use super::HttpVersion;
use crate::http::headers::Headers;

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
    Patch,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method = match self {
            HttpMethod::Get => "GET",
            HttpMethod::Head => "HEAD",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Connect => "CONNECT",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Patch => "PATCH",
        };
        write!(f, "{method}")
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    method: HttpMethod,
    path: String,
    version: HttpVersion,
    headers: Headers,
}

impl HttpRequest {
    pub fn new<P: Into<String>>(method: HttpMethod, path: P, version: HttpVersion) -> Self {
        HttpRequest {
            method,
            path: path.into(),
            version,
            headers: Default::default(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }

    pub fn add_header<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.headers.add(UniCase::new(key.into()), value.into());
    }

    pub fn get_header<K>(&self, key: &K) -> Option<&str>
    where
        K: ?Sized,
        K: AsRef<str>,
    {
        self.headers
            .get(&UniCase::new(key.as_ref().to_string()))
    }

    pub fn delete_header<K>(&mut self, key: &K) -> Option<String>
    where
        K: ?Sized,
        K: AsRef<str>,
    {
        self.headers.remove(&UniCase::new(key.as_ref().to_string()))
    }
}

impl fmt::Display for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}\r\n", self.method, self.path, self.version)?;
        for (key, value) in &self.headers {
            write!(f, "{key}: {value}\r\n")?;
        }
        write!(f, "\r\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_http_method() {
        assert_eq!(HttpMethod::Get.to_string(), "GET");
        assert_eq!(HttpMethod::Head.to_string(), "HEAD");
        assert_eq!(HttpMethod::Post.to_string(), "POST");
        assert_eq!(HttpMethod::Put.to_string(), "PUT");
        assert_eq!(HttpMethod::Delete.to_string(), "DELETE");
        assert_eq!(HttpMethod::Connect.to_string(), "CONNECT");
        assert_eq!(HttpMethod::Options.to_string(), "OPTIONS");
        assert_eq!(HttpMethod::Trace.to_string(), "TRACE");
        assert_eq!(HttpMethod::Patch.to_string(), "PATCH");
    }

    #[test]
    fn creates_standard_request() {
        let _ = HttpRequest::new(HttpMethod::Get, "/", HttpVersion::Version1_1);
    }

    #[test]
    fn create_requests_for_any_method() {
        let _ = HttpRequest::new(HttpMethod::Get, "/", HttpVersion::Version1_1);
        let _ = HttpRequest::new(HttpMethod::Head, "/", HttpVersion::Version1_1);
        let _ = HttpRequest::new(HttpMethod::Post, "/", HttpVersion::Version1_1);
        let _ = HttpRequest::new(HttpMethod::Put, "/", HttpVersion::Version1_1);
        let _ = HttpRequest::new(HttpMethod::Delete, "/", HttpVersion::Version1_1);
        let _ = HttpRequest::new(HttpMethod::Connect, "/", HttpVersion::Version1_1);
        let _ = HttpRequest::new(HttpMethod::Options, "/", HttpVersion::Version1_1);
        let _ = HttpRequest::new(HttpMethod::Trace, "/", HttpVersion::Version1_1);
        let _ = HttpRequest::new(HttpMethod::Patch, "/", HttpVersion::Version1_1);
    }

    #[test]
    fn create_requests_for_any_version() {
        let _ = HttpRequest::new(HttpMethod::Get, "/", HttpVersion::Version0_9);
        let _ = HttpRequest::new(HttpMethod::Get, "/", HttpVersion::Version1_0);
        let _ = HttpRequest::new(HttpMethod::Get, "/", HttpVersion::Version1_1);
        let _ = HttpRequest::new(HttpMethod::Get, "/", HttpVersion::Version2_0);
    }

    #[test]
    fn can_add_headers() {
        let mut req = HttpRequest::new(HttpMethod::Get, "/", HttpVersion::Version1_1);
        req.add_header("my header", "my value");
        req.add_header("my header 2", "my value 2");
        assert_eq!(req.get_header("my header"), Some("my value"));
        assert_eq!(req.get_header("my header 2"), Some("my value 2"));
        assert_eq!(req.get_header("non existant"), None);
    }

    #[test]
    fn can_delete_headers() {
        let mut req = HttpRequest::new(HttpMethod::Get, "/", HttpVersion::Version1_1);
        req.add_header("my header", "my value");
        req.add_header("my header 2", "my value 2");
        assert_eq!(req.get_header("my header"), Some("my value"));
        assert_eq!(req.get_header("my header 2"), Some("my value 2"));
        assert_eq!(req.get_header("non existant"), None);

        assert_eq!(
            req.delete_header("my header 2"),
            Some("my value 2".to_string())
        );
        assert_eq!(req.get_header("my header 2"), None);
        assert_eq!(req.delete_header("my header 2"), None);
        assert_eq!(req.delete_header("non existant"), None);
    }

    #[test]
    fn serializes_request() {
        let mut req = HttpRequest::new(HttpMethod::Get, "/my_path.html", HttpVersion::Version1_1);
        req.add_header("my header", "my value");
        req.add_header("my header 2", "my value 2");

        let req_serialized = req.to_string();

        // Rust HashMap iteration order is only consistent with one hashmap during the same run so we have to check both orders
        assert!(
            (req_serialized == "GET /my_path.html HTTP/1.1\r\nmy header 2: my value 2\r\nmy header: my value\r\n\r\n")
            || (req_serialized ==  "GET /my_path.html HTTP/1.1\r\nmy header: my value\r\nmy header 2: my value 2\r\n\r\n")
        );
    }

    #[test]
    fn headers_are_case_insensitive() {
        let mut req = HttpRequest::new(HttpMethod::Get, "/index.html", HttpVersion::Version1_1);
        req.add_header("My Header", "My Value");
        let value = req.get_header("my header").expect("Couldn't get value");
        assert_eq!(value, "My Value");
    }
}
