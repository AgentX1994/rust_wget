use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::TcpStream,
    time::Duration,
};

use rust_wget::http::{HttpMethod, HttpRequest, HttpResponse, HttpStatus, HttpVersion};

// TODO error handling
#[derive(Debug, PartialEq)]
enum Protocol {
    // TODO allow https?
    Http,
}

#[derive(Debug, PartialEq)]
struct ParsedUrl {
    protocol: Protocol,
    domain_name: String,
    path: String,
}

impl ParsedUrl {
    fn parse(url: &str) -> Self {
        let mut parts = url.split('/');
        let protocol_str = parts.next().expect("Invalid url, unable to read protocol");
        let _ = parts.next();
        let domain_name = parts
            .next()
            .expect("Invalid url, unable to read domain name")
            .to_string();
        let path = parts.collect::<Vec<&str>>().join("/");
        let path = format!("/{}", path);
        let protocol = match protocol_str {
            "http:" => Protocol::Http,
            _ => panic!("Unknown protocol {}", protocol_str),
        };
        ParsedUrl {
            protocol,
            domain_name,
            path,
        }
    }
}

fn read_http_line(reader: &mut BufReader<TcpStream>) -> io::Result<String> {
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

fn fetch_url(url: &str) -> io::Result<HttpResponse> {
    let parsed_url = ParsedUrl::parse(url);
    println!("{:?}", parsed_url);
    assert!(parsed_url.protocol == Protocol::Http);
    let mut socket = TcpStream::connect((&parsed_url.domain_name[..], 80u16))?;

    let mut request = HttpRequest::new(HttpMethod::Get, parsed_url.path, HttpVersion::Version1_1);
    request.add_header("Host", parsed_url.domain_name);
    request.add_header("User-Agent", "Wget/1.21.3");
    request.add_header("Accept", "*/*");
    request.add_header("Accept-Encoding", "identity");
    request.add_header("Connection", "Keep-Alive");

    let request_serialized = request.serialize();
    println!(
        "------ request start ------\n{}\n------ request end -----",
        request
    );
    socket.write(&request_serialized)?;

    socket.set_read_timeout(Some(Duration::from_secs(30)))?;

    let mut socket_reader = BufReader::new(socket);

    let mut response = {
        let line = read_http_line(&mut socket_reader)?;
        let mut line_split = line.split(" ");

        let version_str = line_split.next().expect("No Version in response");
        let version = HttpVersion::try_from(version_str).expect("Unable to determine HTTP version");

        let status_code_str = line_split.next().expect("No status code");
        let status_code = status_code_str
            .parse::<HttpStatus>()
            .expect("Unable to detect status code");

        let status_message = line_split.next().expect("No status message");

        HttpResponse::new(version, status_code, status_message.to_string())
    };

    loop {
        let line = read_http_line(&mut socket_reader)?;
        if line.is_empty() { break; }
        let mut line_split = line.split(": ");
        let key = line_split.next().expect("No header key");
        let value = line_split.next().expect("No header value");
        response.add_header(key, value);
    }

    if let Some(len_str) = response.get_header("Content-Length") {
        let length = len_str.parse::<usize>().expect("Invalid content length");
        let mut buf = vec![0; length];
        socket_reader.read_exact(&mut buf)?;
        response.set_data(buf);
    }

    Ok(response)
}

fn main() {
    let mut args = std::env::args();
    let prog_name = args.next().unwrap_or(env!("CARGO_PKG_NAME").to_string());
    let urls: Vec<String> = args.collect();
    if urls.is_empty() {
        eprintln!("Usage: {} <URL> [<URL> ...]", prog_name);
        std::process::exit(1);
    }
    let mut has_error = false;
    for url in urls {
        let contents = fetch_url(&url);
        match contents {
            Ok(response) => {
                println!(
                    "------ response start ------\n{}\n------ response end -----",
                    response
                );
            }
            Err(e) => {
                eprintln!("{:?}", e);
                has_error = true;
            }
        }
    }
    if has_error {
        std::process::exit(1);
    }
}
