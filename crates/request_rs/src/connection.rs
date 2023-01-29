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
            println!("Connecting to {domain} port {port}");
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
                "------ request start ------\n{request}\n------ request end -----"
            );
        }
        self.socket.write_all(&request.serialize())?;

        let mut reader = BufReader::new(&mut self.socket);

        HttpResponse::receive_response(&mut reader, config)
    }

    pub fn get_domain(&self) -> &str {
        &self.domain
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_socket(&self) -> &TcpStream {
        &self.socket
    }

    pub fn get_socket_mut(&mut self) -> &mut TcpStream {
        &mut self.socket
    }
}

#[cfg(test)]
mod tests {
    use crate::http::HttpStatusCode;

    use super::*;

    use std::hint;
    use std::io::BufRead;
    use std::net::TcpListener;
    use std::sync::atomic::AtomicU16;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::thread;
    use std::thread::JoinHandle;

    fn get_listener_port(listener: &TcpListener) -> u16 {
        listener
            .local_addr()
            .expect("Listener has no local addr!")
            .port()
    }

    fn create_listener_thread<F>(mut on_incoming: F) -> (u16, JoinHandle<()>)
    where
        F: FnMut(TcpStream) + Send + 'static,
    {
        let port_atomic = Arc::new(AtomicU16::new(0));
        let port_atomic_t = port_atomic.clone();
        let t = thread::spawn(move || {
            let listener = TcpListener::bind("localhost:0").expect("Could not create listener");
            let port = get_listener_port(&listener);
            port_atomic_t.store(port, Ordering::Relaxed);

            for conn in listener.incoming() {
                on_incoming(conn.expect("Error in incoming"));
            }
        });

        let port;
        loop {
            let port_num = port_atomic.load(Ordering::Relaxed);
            if port_num != 0 {
                port = port_num;
                break;
            }
            hint::spin_loop()
        }

        (port, t)
    }

    #[test]
    fn can_create_connection() {
        let (port, _l_thread) = create_listener_thread(|_s| {});
        let config = Configuration { debug: 0 };
        let _conn = Connection::new("localhost".to_string(), port, &config)
            .expect("Could not create connection");
    }

    #[test]
    fn can_send_request() {
        let (port, _l_thread) = create_listener_thread(|mut s| {
            let mut response = HttpResponse::new(
                HttpVersion::Version1_1,
                HttpStatusCode::Ok,
                "Ok".to_string(),
            );
            response.add_header("My Header", "Value");

            let mut reader = BufReader::new(&mut s);
            {
                let mut line = String::new();
                reader.read_line(&mut line).expect("Could not read line");
                assert_eq!(line.trim(), "GET / HTTP/1.1");
            }

            let mut request = HttpRequest::new(HttpMethod::Get, "/", HttpVersion::Version1_1);
            for _ in 0..5 {
                let mut line = String::new();
                reader.read_line(&mut line).expect("Could not read line");
                let index = line.find(':').expect("No colon in recieved line!");
                let (key, value) = line.split_at(index);
                request.add_header(key.trim(), value[1..].trim());
            }

            println!("{request}");

            assert_eq!(request.get_header("Host"), Some("localhost"));
            assert_eq!(request.get_header("User-Agent"), Some("Wget/1.21.3"));
            assert_eq!(request.get_header("Accept"), Some("*/*"));
            assert_eq!(request.get_header("Accept-Encoding"), Some("identity"));
            assert_eq!(request.get_header("Connection"), Some("Keep-Alive"));

            s.write_all(&response.serialize())
                .expect("Could not write response");
        });
        let config = Configuration { debug: 0 };
        let mut conn = Connection::new("localhost".to_string(), port, &config)
            .expect("Could not create connection");
        let resp = conn
            .send_request("/", &config)
            .expect("Could not receive response");
        assert_eq!(resp.version, HttpVersion::Version1_1);
        assert_eq!(resp.status_code, HttpStatusCode::Ok);
        assert_eq!(resp.status_message, "Ok");
        assert_eq!(resp.get_header("My Header"), Some("Value"));
    }
}
