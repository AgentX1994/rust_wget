use std::{
    fs::File,
    io::{self, BufReader, Write},
    net::TcpStream,
    time::Duration,
};

use clap::Parser;

use rust_wget::{
    error::WgetResult,
    http::{
        HttpConnectionCache, HttpMethod, HttpRequest, HttpResponse, HttpStatusFamily, HttpVersion,
    },
    protocol::Protocol,
    url::ParsedUrl,
    Configuration,
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

fn fetch_url(
    url: &ParsedUrl,
    socket: &mut TcpStream,
    config: &Configuration,
) -> WgetResult<HttpResponse> {
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

fn main() -> anyhow::Result<()> {
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
        .map(|path| {
            if path == "-" {
                if config.debug > 0 {
                    println!("Writing to stdout");
                }
                Ok(Box::new(io::stdout()) as Box<dyn io::Write>)
            } else {
                if config.debug > 0 {
                    println!("Writing to {}", path);
                }
                File::create(path).map(|f| Box::new(f) as Box<dyn io::Write>)
            }
        })
        // Go from Option<Result<...>> to Result<Option<...>>
        .map_or(Ok(None), |r| r.map(Some))?;
    let mut connection_cache = HttpConnectionCache::default();
    for url in options.urls {
        let mut current_url = url;
        let mut successful = false;
        while !successful {
            let parsed_url = ParsedUrl::parse(&current_url, &config)?;
            if config.debug > 0 {
                println!("{:?}", parsed_url);
            }
            if parsed_url.protocol != Protocol::Http {
                return Err(anyhow::anyhow!(
                    "Protocols other than HTTP are not yet implemented"
                ));
            }
            let socket = connection_cache.get_connection(&parsed_url, &config)?;
            let result = fetch_url(&parsed_url, socket, &config);
            match result {
                Ok(response) => {
                    if config.debug > 0 {
                        println!(
                            "------ response start ------\n{}\n------ response end -----",
                            response
                        );
                    }
                    match response.status_family() {
                        HttpStatusFamily::Successful => {
                            successful = true;
                            if let Some(output_file) = &mut output_file {
                                if let Err(e) = output_file.write_all(response.get_data()) {
                                    eprintln!("Could not write data to output file: {}", e);
                                }
                            } else if let Err(e) =
                                File::create(parsed_url.filename)?.write_all(response.get_data())
                            {
                                eprintln!("Could not write data to output file: {}", e);
                            }
                        }
                        HttpStatusFamily::Redirection => {
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
                        HttpStatusFamily::Informational => {
                            eprintln!("Received Informational response?");
                            let bytes = response.serialize();
                            let response_string = String::from_utf8_lossy(&bytes);
                            println!("{}", response_string);
                            successful = true;
                        }
                        HttpStatusFamily::ClientError => {
                            eprintln!("ServerError!");
                            let bytes = response.serialize();
                            let response_string = String::from_utf8_lossy(&bytes);
                            println!("{}", response_string);
                            successful = true;
                        }
                        HttpStatusFamily::ServerError => {
                            eprintln!("ServerError!");
                            let bytes = response.serialize();
                            let response_string = String::from_utf8_lossy(&bytes);
                            println!("{}", response_string);
                            successful = true;
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

    Ok(())
}
