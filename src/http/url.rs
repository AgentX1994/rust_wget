use crate::http::common::Protocol;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: Protocol,
    pub domain_name: String,
    pub port: u16,
    pub path: String,
    pub filename: String,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Self {
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
        let path = parts.collect::<Vec<&str>>();
        let filename = path.last().unwrap_or(&"index.html").to_string();
        let path = path.join("/");
        let path = format!("/{}", path);
        ParsedUrl {
            protocol,
            domain_name,
            port,
            path,
            filename,
        }
    }
}
