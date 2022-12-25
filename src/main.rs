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

impl Protocol {
    pub fn get_port(&self) -> u16 {
        match self {
            Protocol::Http => 80,
        }
    }
}

#[derive(Debug, PartialEq)]
struct ParsedUrl {
    protocol: Protocol,
    domain_name: String,
    port: u16,
    path: String,
}

impl ParsedUrl {
    fn parse(url: &str) -> Self {
        let mut parts = url.split('/');
        let protocol_str = parts.next().expect("Invalid url, unable to read protocol");
        let protocol = match protocol_str {
            "http:" => Protocol::Http,
            _ => panic!("Unknown protocol {}", protocol_str),
        };
        let _ = parts.next();
        let (domain_name, port) = {
            let domain_name_port = parts
                .next()
                .expect("Invalid url, unable to read domain name")
                .to_string();
            if let Some(loc) = domain_name_port.rfind(':') {
                let (domain_name, port_str) = domain_name_port.split_at(loc);
                let port_str = &port_str[1..]; // remove colon
                (
                    domain_name.to_string(),
                    port_str.parse::<u16>().expect("Invalid port"),
                )
            } else {
                (domain_name_port, protocol.get_port())
            }
        };
        let path = parts.collect::<Vec<&str>>().join("/");
        let path = format!("/{}", path);
        ParsedUrl {
            protocol,
            domain_name,
            port,
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
    let mut socket = TcpStream::connect((&parsed_url.domain_name[..], parsed_url.port))?;

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
        if line.is_empty() {
            break;
        }
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
    } else if let Some("chunked") = response.get_header("Transfer-Encoding") {
        let mut data: Vec<u8> = Vec::new();
        loop {
            let len_str = read_http_line(&mut socket_reader)?;
            let length: usize = usize::from_str_radix(&len_str, 16).expect("Invalid chunk length");
            if length == 0 {
                break;
            }
            let mut chunk = vec![0u8; length];
            socket_reader.read_exact(&mut chunk)?;
            data.extend(chunk);
            let mut ending = [0u8, 0u8];
            socket_reader.read_exact(&mut ending)?;
            if &ending != b"\r\n" {
                panic!("Invalid chunk ending");
            }
        }
        response.set_data(data);
        // TODO parse trailers?
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
        let mut current_url = url;
        let mut successful = false;
        while !successful {
            let result = fetch_url(&current_url);
            match result {
                Ok(response) => {
                    println!(
                        "------ response start ------\n{}\n------ response end -----",
                        response
                    );
                    match response.status_code {
                        HttpStatus::Ok => successful = true,
                        HttpStatus::MovedPermanently => {
                            if let Some(new_url) = response.get_header("Location") {
                                println!(
                                    "Got {} with Location \"{}\"",
                                    response.status_code, new_url
                                );
                                current_url = new_url.to_string();
                            } else {
                                eprintln!("Got {} without a Location!", response.status_code);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{:?}", e);
                    has_error = true;
                    break;
                }
            }
        }
    }
    if has_error {
        std::process::exit(1);
    }
}
