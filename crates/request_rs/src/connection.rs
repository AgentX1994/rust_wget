use std::{
    io::{BufReader, Write},
    net::TcpStream,
    time::Duration,
};

use crate::{
    error::WgetResult,
    http::{HttpMethod, HttpRequest, HttpResponse, HttpVersion},
    Configuration,
};

#[derive(Debug)]
pub struct Connection {
    domain: String,
    port: u16,
    socket: TcpStream,
}

impl Connection {
    pub fn new(domain: String, port: u16, config: &Configuration) -> WgetResult<Self> {
        if config.debug > 1 {
            println!("Connecting to {} port {}", domain, port);
        }
        let socket = TcpStream::connect((&domain[..], port))?;
        socket.set_read_timeout(Some(Duration::from_secs(30)))?;
        Ok(Self {
            domain,
            port,
            socket,
        })
    }

    pub fn send_request(&mut self, path: &str, config: &Configuration) -> WgetResult<HttpResponse> {
        let mut request = HttpRequest::new(HttpMethod::Get, path, HttpVersion::Version1_1);
        request.add_header("Host", &self.domain);
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
        self.socket.write_all(&request.serialize())?;

        let mut reader = BufReader::new(&mut self.socket);

        HttpResponse::receive_response(&mut reader, config)
    }

    pub fn get_socket(&self) -> &TcpStream {
        &self.socket
    }

    pub fn get_socket_mut(&mut self) -> &mut TcpStream {
        &mut self.socket
    }
}
