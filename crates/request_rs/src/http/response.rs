use std::{
    collections::HashMap,
    fmt,
    io::{self, BufRead},
    str::FromStr,
};

use unicase::UniCase;

use crate::{
    error::{WgetError, WgetResult},
    Configuration,
};

use super::HttpVersion;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HttpStatusFamily {
    Informational,
    Successful,
    Redirection,
    ClientError,
    ServerError,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum HttpStatusCode {
    Continue = 100,
    SwitchingProtocols = 101,
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    UriTooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418,
    UnprocessableEntity = 422,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HttpVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,
}

impl HttpStatusCode {
    pub fn family(&self) -> HttpStatusFamily {
        let code = *self as u16;
        match code {
            100..=199 => HttpStatusFamily::Informational,
            200..=299 => HttpStatusFamily::Successful,
            300..=399 => HttpStatusFamily::Redirection,
            400..=499 => HttpStatusFamily::ClientError,
            500..=599 => HttpStatusFamily::ServerError,
            _ => unreachable!("HttpStatusCode was outside any family!"),
        }
    }
}

impl FromStr for HttpStatusCode {
    type Err = WgetError;

    fn from_str(s: &str) -> WgetResult<Self> {
        let status_code = s
            .parse::<u16>()
            .map_err(|_| WgetError::ParsingError(format!("Status Code not a number: {}", s)))?;

        status_code.try_into()
    }
}

impl TryFrom<u16> for HttpStatusCode {
    type Error = WgetError;

    fn try_from(value: u16) -> WgetResult<Self> {
        match value {
            100 => Ok(HttpStatusCode::Continue),
            101 => Ok(HttpStatusCode::SwitchingProtocols),
            200 => Ok(HttpStatusCode::Ok),
            201 => Ok(HttpStatusCode::Created),
            202 => Ok(HttpStatusCode::Accepted),
            203 => Ok(HttpStatusCode::NonAuthoritativeInformation),
            204 => Ok(HttpStatusCode::NoContent),
            205 => Ok(HttpStatusCode::ResetContent),
            206 => Ok(HttpStatusCode::PartialContent),
            300 => Ok(HttpStatusCode::MultipleChoices),
            301 => Ok(HttpStatusCode::MovedPermanently),
            302 => Ok(HttpStatusCode::Found),
            303 => Ok(HttpStatusCode::SeeOther),
            304 => Ok(HttpStatusCode::NotModified),
            307 => Ok(HttpStatusCode::TemporaryRedirect),
            308 => Ok(HttpStatusCode::PermanentRedirect),
            400 => Ok(HttpStatusCode::BadRequest),
            401 => Ok(HttpStatusCode::Unauthorized),
            402 => Ok(HttpStatusCode::PaymentRequired),
            403 => Ok(HttpStatusCode::Forbidden),
            404 => Ok(HttpStatusCode::NotFound),
            405 => Ok(HttpStatusCode::MethodNotAllowed),
            406 => Ok(HttpStatusCode::NotAcceptable),
            407 => Ok(HttpStatusCode::ProxyAuthenticationRequired),
            408 => Ok(HttpStatusCode::RequestTimeout),
            409 => Ok(HttpStatusCode::Conflict),
            410 => Ok(HttpStatusCode::Gone),
            411 => Ok(HttpStatusCode::LengthRequired),
            412 => Ok(HttpStatusCode::PreconditionFailed),
            413 => Ok(HttpStatusCode::PayloadTooLarge),
            414 => Ok(HttpStatusCode::UriTooLong),
            415 => Ok(HttpStatusCode::UnsupportedMediaType),
            416 => Ok(HttpStatusCode::RangeNotSatisfiable),
            417 => Ok(HttpStatusCode::ExpectationFailed),
            418 => Ok(HttpStatusCode::ImATeapot),
            422 => Ok(HttpStatusCode::UnprocessableEntity),
            425 => Ok(HttpStatusCode::TooEarly),
            426 => Ok(HttpStatusCode::UpgradeRequired),
            428 => Ok(HttpStatusCode::PreconditionRequired),
            429 => Ok(HttpStatusCode::TooManyRequests),
            431 => Ok(HttpStatusCode::RequestHeaderFieldsTooLarge),
            451 => Ok(HttpStatusCode::UnavailableForLegalReasons),
            500 => Ok(HttpStatusCode::InternalServerError),
            501 => Ok(HttpStatusCode::NotImplemented),
            502 => Ok(HttpStatusCode::BadGateway),
            503 => Ok(HttpStatusCode::ServiceUnavailable),
            504 => Ok(HttpStatusCode::GatewayTimeout),
            505 => Ok(HttpStatusCode::HttpVersionNotSupported),
            506 => Ok(HttpStatusCode::VariantAlsoNegotiates),
            507 => Ok(HttpStatusCode::InsufficientStorage),
            508 => Ok(HttpStatusCode::LoopDetected),
            510 => Ok(HttpStatusCode::NotExtended),
            511 => Ok(HttpStatusCode::NetworkAuthenticationRequired),
            _ => Err(WgetError::InvalidStatusCode(value)),
        }
    }
}

impl fmt::Display for HttpStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u16)
    }
}

fn read_http_line<S>(reader: &mut S) -> io::Result<String>
where
    S: BufRead,
{
    let mut line = String::new();
    reader.read_line(&mut line)?;
    if let Some('\n') = line.chars().last() {
        line.pop();
    }
    if let Some('\r') = line.chars().last() {
        line.pop();
    }
    Ok(line)
}

#[derive(Debug)]
pub struct HttpResponse {
    pub version: HttpVersion,
    pub status_code: HttpStatusCode,
    pub status_message: String,
    headers: HashMap<UniCase<String>, String>,
    data: Vec<u8>,
}

impl HttpResponse {
    pub fn new(version: HttpVersion, status_code: HttpStatusCode, status_message: String) -> Self {
        HttpResponse {
            version,
            status_code,
            status_message,
            headers: HashMap::new(),
            data: Vec::new(),
        }
    }

    pub fn status_family(&self) -> HttpStatusFamily {
        self.status_code.family()
    }

    pub fn add_header<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.headers.insert(UniCase::new(key.into()), value.into());
    }

    pub fn set_data<D: Into<Vec<u8>>>(&mut self, data: D) {
        self.data = data.into();
    }

    pub fn get_header<K>(&self, key: &K) -> Option<&str>
    where
        K: ?Sized,
        K: AsRef<str>,
    {
        self.headers
            .get(&UniCase::new(key.as_ref().to_string()))
            .map(|s| &**s)
    }

    pub fn delete_header<K>(&mut self, key: &K) -> Option<String>
    where
        K: ?Sized,
        K: AsRef<str>,
    {
        self.headers.remove(&UniCase::new(key.as_ref().to_string()))
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }

    pub fn receive_response<S>(mut socket: &mut S, config: &Configuration) -> WgetResult<Self>
    where
        S: BufRead,
    {
        let mut response = {
            let line = read_http_line(&mut socket)?;
            if config.debug > 1 {
                println!("Read status line: {}", &line);
            }
            let mut line_split = line.split(' ');

            let version_str = line_split
                .next()
                .ok_or_else(|| WgetError::ParsingError("No Version in response".to_string()))?;
            let version = HttpVersion::try_from(version_str)?;

            let status_code_str = line_split
                .next()
                .ok_or_else(|| WgetError::ParsingError("No status code".to_string()))?;
            let status_code = status_code_str.parse::<HttpStatusCode>()?;

            let status_message = line_split.collect::<Vec<_>>().join(" ");

            HttpResponse::new(version, status_code, status_message)
        };

        loop {
            let line = read_http_line(&mut socket)?;
            if line.is_empty() {
                if config.debug > 1 {
                    println!("Finished reading headers");
                }
                break;
            }
            if config.debug > 1 {
                println!("Read header line: {}", &line);
            }
            let mut line_split = line.split(": ");
            let key = line_split
                .next()
                .ok_or_else(|| WgetError::ParsingError("No header key".to_string()))?;
            let value = line_split
                .next()
                .ok_or_else(|| WgetError::ParsingError("No header value".to_string()))?;
            response.add_header(key, value);
        }

        if let Some(len_str) = response.get_header("Content-Length") {
            let length = len_str.parse::<usize>().map_err(|_| {
                WgetError::ParsingError(format!("Invalid content length {}", len_str))
            })?;
            if config.debug > 1 {
                println!("receiving normal file of length {}", length);
            }
            let mut buf = vec![0; length];
            socket.read_exact(&mut buf)?;
            response.set_data(buf);
        } else if let Some("chunked") = response.get_header("Transfer-Encoding") {
            let mut data: Vec<u8> = Vec::new();
            loop {
                let len_str = read_http_line(&mut socket)?;
                if config.debug > 1 {
                    println!("receiving chunk of length 0x{}", len_str);
                }
                let length: usize = usize::from_str_radix(&len_str, 16).map_err(|_| {
                    WgetError::ParsingError(format!("Invalid chunk length {}", len_str))
                })?;
                if length == 0 {
                    break;
                }
                let mut chunk = vec![0u8; length];
                socket.read_exact(&mut chunk)?;
                data.extend(chunk);
                let mut ending = [0u8, 0u8];
                socket.read_exact(&mut ending)?;
                if &ending != b"\r\n" {
                    panic!("Invalid chunk ending");
                }
            }
            if config.debug > 1 {
                println!("All chunks received");
            }
            response.set_data(data);
            // TODO parse trailers?
        }

        Ok(response)
    }
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}\r\n",
            self.version, self.status_code, self.status_message
        )?;
        for (key, value) in &self.headers {
            write!(f, "{}: {}\r\n", key, value)?;
        }
        write!(f, "{}\r\n\r\n", String::from_utf8_lossy(&self.data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_http_status() {
        assert!(matches!("200".parse(), Ok(HttpStatusCode::Ok)));
        assert!(matches!(
            "301".parse(),
            Ok(HttpStatusCode::MovedPermanently)
        ));
        assert!(matches!(
            "Not a status".parse::<HttpStatusCode>(),
            Err(WgetError::ParsingError(_))
        ));
    }

    #[test]
    fn can_display_http_status() {
        assert_eq!(HttpStatusCode::Ok.to_string(), "200");
        assert_eq!(HttpStatusCode::MovedPermanently.to_string(), "301");
    }

    #[test]
    fn can_determine_status_family() {
        assert_eq!(
            HttpStatusCode::Continue.family(),
            HttpStatusFamily::Informational
        );
        assert_eq!(
            HttpStatusCode::SwitchingProtocols.family(),
            HttpStatusFamily::Informational
        );
        assert_eq!(HttpStatusCode::Ok.family(), HttpStatusFamily::Successful);
        assert_eq!(
            HttpStatusCode::Accepted.family(),
            HttpStatusFamily::Successful
        );
        assert_eq!(
            HttpStatusCode::MovedPermanently.family(),
            HttpStatusFamily::Redirection
        );
        assert_eq!(
            HttpStatusCode::NotModified.family(),
            HttpStatusFamily::Redirection
        );
        assert_eq!(
            HttpStatusCode::MethodNotAllowed.family(),
            HttpStatusFamily::ClientError
        );
        assert_eq!(
            HttpStatusCode::ImATeapot.family(),
            HttpStatusFamily::ClientError
        );
        assert_eq!(
            HttpStatusCode::NotImplemented.family(),
            HttpStatusFamily::ServerError
        );
        assert_eq!(
            HttpStatusCode::NetworkAuthenticationRequired.family(),
            HttpStatusFamily::ServerError
        );
    }

    #[test]
    fn can_construct_http_response() {
        let _ = HttpResponse::new(
            HttpVersion::Version0_9,
            HttpStatusCode::Ok,
            "Ok".to_string(),
        );
    }

    #[test]
    fn can_add_headers() {
        let mut resp = HttpResponse::new(
            HttpVersion::Version1_1,
            HttpStatusCode::Ok,
            "Ok".to_string(),
        );
        resp.add_header("my header", "my value");
        resp.add_header("my header 2", "my value 2");
        assert_eq!(resp.get_header("my header"), Some("my value"));
        assert_eq!(resp.get_header("my header 2"), Some("my value 2"));
        assert_eq!(resp.get_header("non existant"), None);
    }

    #[test]
    fn can_delete_headers() {
        let mut resp = HttpResponse::new(
            HttpVersion::Version1_1,
            HttpStatusCode::Ok,
            "Ok".to_string(),
        );
        resp.add_header("my header", "my value");
        resp.add_header("my header 2", "my value 2");
        assert_eq!(resp.get_header("my header"), Some("my value"));
        assert_eq!(resp.get_header("my header 2"), Some("my value 2"));
        assert_eq!(resp.get_header("non existant"), None);

        assert_eq!(
            resp.delete_header("my header 2"),
            Some("my value 2".to_string())
        );
        assert_eq!(resp.get_header("my header 2"), None);
        assert_eq!(resp.delete_header("my header 2"), None);
        assert_eq!(resp.delete_header("non existant"), None);
    }

    #[test]
    fn can_construct_http_response_with_version() {
        let _ = HttpResponse::new(
            HttpVersion::Version0_9,
            HttpStatusCode::Ok,
            "Ok".to_string(),
        );
        let _ = HttpResponse::new(
            HttpVersion::Version1_0,
            HttpStatusCode::Ok,
            "Ok".to_string(),
        );
        let _ = HttpResponse::new(
            HttpVersion::Version1_1,
            HttpStatusCode::Ok,
            "Ok".to_string(),
        );
        let _ = HttpResponse::new(
            HttpVersion::Version2_0,
            HttpStatusCode::Ok,
            "Ok".to_string(),
        );
    }

    #[test]
    fn can_construct_http_response_with_status() {
        let _ = HttpResponse::new(
            HttpVersion::Version0_9,
            HttpStatusCode::Ok,
            "Ok".to_string(),
        );
        let _ = HttpResponse::new(
            HttpVersion::Version1_0,
            HttpStatusCode::MovedPermanently,
            "Ok".to_string(),
        );
    }

    #[test]
    fn can_read_response() {
        let mut sample_response = "HTTP/1.1 200 Ok\r\nmy header: my value\r\nmy header 2: my value 2\r\nContent-Length: 5\r\n\r\nabcde".as_bytes();
        let config = Configuration { debug: 0 };

        let response = HttpResponse::receive_response(&mut sample_response, &config)
            .expect("Could not read response!");

        assert_eq!(response.version, HttpVersion::Version1_1);
        assert_eq!(response.status_code, HttpStatusCode::Ok);
        assert_eq!(response.status_message, "Ok");
        assert_eq!(response.get_data(), "abcde".as_bytes());
        assert_eq!(response.get_header("my header"), Some("my value"));
        assert_eq!(response.get_header("my header 2"), Some("my value 2"));
        assert_eq!(response.get_header("Content-Length"), Some("5"));
        assert_eq!(response.get_header("not a key"), None);
    }

    #[test]
    fn headers_are_case_insensitive() {
        let mut res = HttpResponse::new(
            HttpVersion::Version1_1,
            HttpStatusCode::Ok,
            "Ok".to_string(),
        );
        res.add_header("My Header", "My Value");
        let value = res.get_header("my header").expect("Couldn't get value");
        assert_eq!(value, "My Value");
    }
}
