use std::{collections::HashMap, io, net::TcpStream};

use super::{Configuration, ParsedUrl};

#[derive(Debug, Default)]
pub struct HttpConnectionCache {
    connections: HashMap<(String, u16), TcpStream>,
}

impl HttpConnectionCache {
    pub fn get_connection(
        &mut self,
        url: &ParsedUrl,
        config: &Configuration,
    ) -> io::Result<&mut TcpStream> {
        match self.connections.entry((url.domain_name.clone(), url.port)) {
            std::collections::hash_map::Entry::Occupied(o) => {
                if config.debug > 1 {
                    println!(
                        "Reusing old connection for {} port {}",
                        url.domain_name, url.port
                    );
                }
                Ok(o.into_mut())
            }
            std::collections::hash_map::Entry::Vacant(v) => {
                if config.debug > 1 {
                    println!("Connecting to {} port {}", url.domain_name, url.port);
                }
                let socket = TcpStream::connect((url.domain_name.as_str(), url.port))?;
                Ok(v.insert(socket))
            }
        }
    }
}
