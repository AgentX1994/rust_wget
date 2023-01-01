use std::{
    io::{self, BufReader, Write},
    net::TcpStream,
    time::Duration,
};

use rust_wget::http::{
    HttpMethod, HttpRequest, HttpResponse, HttpStatus, HttpVersion, ParsedUrl, Protocol,
};

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

    println!(
        "------ request start ------\n{}\n------ request end -----",
        request
    );
    socket.write(&request.serialize())?;

    socket.set_read_timeout(Some(Duration::from_secs(30)))?;

    let mut socket_reader = BufReader::new(socket);

    HttpResponse::receive_response(&mut socket_reader)
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
