use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt,
    hash::Hash,
    io::{self, BufRead},
    str::FromStr,
};

use super::{Configuration, HttpVersion};

#[derive(Debug, PartialEq, Eq)]
pub enum HttpStatus {
    Ok,
    MovedPermanently,
}

impl FromStr for HttpStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let status_code = s.parse::<u16>().or(Err(()))?;

        match status_code {
            200 => Ok(HttpStatus::Ok),
            301 => Ok(HttpStatus::MovedPermanently),
            _ => unimplemented!(),
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
    pub status_code: HttpStatus,
    pub status_message: String,
    headers: HashMap<String, String>,
    data: Vec<u8>,
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

    pub fn get_header<K>(&self, k: &K) -> Option<&str>
    where
        K: ?Sized,
        String: Borrow<K>,
        K: Hash + Eq,
    {
        self.headers.get(k).map(|s| &**s)
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }

    pub fn receive_response<S>(mut socket: &mut S, config: &Configuration) -> io::Result<Self>
    where
        S: BufRead,
    {
        let mut response = {
            let line = read_http_line(&mut socket)?;
            if config.debug > 1 {
                println!("Read status line: {}", &line);
            }
            let mut line_split = line.split(' ');

            let version_str = line_split.next().expect("No Version in response");
            let version =
                HttpVersion::try_from(version_str).expect("Unable to determine HTTP version");

            let status_code_str = line_split.next().expect("No status code");
            let status_code = status_code_str
                .parse::<HttpStatus>()
                .expect("Unable to detect status code");

            let status_message = line_split.next().expect("No status message");

            HttpResponse::new(version, status_code, status_message.to_string())
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
            let key = line_split.next().expect("No header key");
            let value = line_split.next().expect("No header value");
            response.add_header(key, value);
        }

        if let Some(len_str) = response.get_header("Content-Length") {
            let length = len_str.parse::<usize>().expect("Invalid content length");
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
                let length: usize =
                    usize::from_str_radix(&len_str, 16).expect("Invalid chunk length");
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
        assert_eq!("200".parse(), Ok(HttpStatus::Ok));
        assert_eq!("301".parse(), Ok(HttpStatus::MovedPermanently));
        assert_eq!("Not a status".parse::<HttpStatus>(), Err(()));
    }

    #[test]
    fn can_display_http_status() {
        assert_eq!(HttpStatus::Ok.to_string(), "200");
        assert_eq!(HttpStatus::MovedPermanently.to_string(), "301");
    }

    #[test]
    fn can_construct_http_response() {
        let _ = HttpResponse::new(HttpVersion::Version0_9, HttpStatus::Ok, "Ok".to_string());
    }

    #[test]
    fn can_construct_http_response_with_version() {
        let _ = HttpResponse::new(HttpVersion::Version0_9, HttpStatus::Ok, "Ok".to_string());
        let _ = HttpResponse::new(HttpVersion::Version1_0, HttpStatus::Ok, "Ok".to_string());
        let _ = HttpResponse::new(HttpVersion::Version1_1, HttpStatus::Ok, "Ok".to_string());
        let _ = HttpResponse::new(HttpVersion::Version2_0, HttpStatus::Ok, "Ok".to_string());
    }

    #[test]
    fn can_construct_http_response_with_status() {
        let _ = HttpResponse::new(HttpVersion::Version0_9, HttpStatus::Ok, "Ok".to_string());
        let _ = HttpResponse::new(
            HttpVersion::Version1_0,
            HttpStatus::MovedPermanently,
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
        assert_eq!(response.status_code, HttpStatus::Ok);
        assert_eq!(response.status_message, "Ok");
        assert_eq!(response.get_data(), "abcde".as_bytes());
        assert_eq!(response.get_header("my header"), Some("my value"));
        assert_eq!(response.get_header("my header 2"), Some("my value 2"));
        assert_eq!(response.get_header("Content-Length"), Some("5"));
        assert_eq!(response.get_header("not a key"), None);
    }
}
