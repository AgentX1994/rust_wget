use std::{
    fs::File,
    io::{self, BufReader, Write},
    net::TcpStream,
    time::Duration,
};

use clap::Parser;

use rust_wget::http::{
    Configuration, HttpMethod, HttpRequest, HttpResponse, HttpStatus, HttpVersion, ParsedUrl,
    Protocol,
};

#[derive(Debug, Parser)]
#[clap(
    name = "rust_wget",
    version = "0.1.0",
    author = "John Asper <johnasper94@gmail.com>",
    about = "A Rust reimplementation of wget"
)]
struct Options {
    /// An optional output file name, to write the fetched documents to, instead of each individual document. If given, all documents will be concatenated together and written to the given path.
    #[arg(short, long)]
    output_file: Option<String>,
    /// The level of debug information to output to stdout, can be used up to three times
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// The URLs to fetch
    urls: Vec<String>,
}

fn fetch_url(url: &ParsedUrl, config: &Configuration) -> io::Result<HttpResponse> {
    let mut socket = TcpStream::connect((&url.domain_name[..], url.port))?;

    let mut request = HttpRequest::new(HttpMethod::Get, &url.path, HttpVersion::Version1_1);
    request.add_header("Host", &url.domain_name);
    request.add_header("User-Agent", "Wget/1.21.3");
    request.add_header("Accept", "*/*");
    request.add_header("Accept-Encoding", "identity");
    request.add_header("Connection", "Keep-Alive");

    if config.debug > 0 {
        println!(
            "------ request start ------\n{}\n------ request end -----",
            request
        );
    }
    socket.write_all(&request.serialize())?;

    socket.set_read_timeout(Some(Duration::from_secs(30)))?;

    let mut socket_reader = BufReader::new(socket);

    HttpResponse::receive_response(&mut socket_reader, config)
}

fn main() {
    let options = Options::parse();
    if options.debug > 0 {
        println!("{:?}", options);
    }
    let mut has_error = false;
    let config = Configuration {
        debug: options.debug,
    };
    let mut output_file = options
        .output_file
        .map(|path| File::create(path).expect("Could not open requested output file!"));
    for url in options.urls {
        let mut current_url = url;
        let mut successful = false;
        while !successful {
            let parsed_url = ParsedUrl::parse(&current_url);
            if config.debug > 0 {
                println!("{:?}", parsed_url);
            }
            assert!(parsed_url.protocol == Protocol::Http);
            let result = fetch_url(&parsed_url, &config);
            match result {
                Ok(response) => {
                    if config.debug > 0 {
                        println!(
                            "------ response start ------\n{}\n------ response end -----",
                            response
                        );
                    }
                    match response.status_code {
                        HttpStatus::Ok => {
                            successful = true;
                            if let Some(output_file) = &mut output_file {
                                if let Err(e) = output_file.write_all(response.get_data()) {
                                    eprintln!("Could not write data to output file: {}", e);
                                }
                            } else if let Err(e) = File::create(parsed_url.filename)
                                .expect("Could not create output file")
                                .write_all(response.get_data())
                            {
                                eprintln!("Could not write data to output file: {}", e);
                            }
                        }
                        HttpStatus::MovedPermanently => {
                            if let Some(new_url) = response.get_header("Location") {
                                if config.debug > 1 {
                                    println!(
                                        "Got {} with Location \"{}\"",
                                        response.status_code, new_url
                                    );
                                }
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
