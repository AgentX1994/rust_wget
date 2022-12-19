use std::{net::{TcpStream}, io::{self, Write, Read}, time::Duration};

// TODO error handling
#[derive(Debug, PartialEq)]
enum Protocol {
    // TODO allow https?
    Http
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
        let domain_name = parts.next().expect("Invalid url, unable to read domain name").to_string();
        let path = parts.collect::<Vec<&str>>().join("/");
        let path = format!("/{}", path);
        let protocol = match protocol_str {
            "http:" => Protocol::Http,
            _ => panic!("Unknown protocol {}", protocol_str),
        };
        ParsedUrl { protocol, domain_name, path  }
    }
}

fn fetch_url(url: &str) -> io::Result<Vec<u8>> {
    let parsed_url = ParsedUrl::parse(url);
    println!("{:?}", parsed_url);
    assert!(parsed_url.protocol == Protocol::Http);
    let mut socket = TcpStream::connect((&parsed_url.domain_name[..], 80u16))?;
    // GET / HTTP/1.1
    // Host: www.google.com
    // User-Agent: Wget/1.21.3
    // Accept: */*
    // Accept-Encoding: identity
    // Connection: Keep-Alive
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: Wget/1.21.3\r\nAccept: */*\r\nAccept-Encoding: identity\r\nConnection: Keep-Alive\r\n\r\n",
        &parsed_url.path, &parsed_url.domain_name);
    println!("------ request start ------\n{}\n------ request end -----", request);
    socket.write(request.as_bytes())?;
    let mut buf = Vec::<u8>::new();
    socket.set_read_timeout(Some(Duration::from_secs(5)))?;
    match socket.read_to_end(&mut buf) {
        Ok(_) => (),
        Err(e) => match e.kind() {
            io::ErrorKind::TimedOut => println!("Timed out..."),
            io::ErrorKind::WouldBlock => println!("Would Block..."),
            _ => return Err(e),
        }
    }
    Ok(buf)
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
            Ok(data) => {
                let response = String::from_utf8_lossy(&data);
                println!("{}", response);
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
