use crate::{
    error::{WgetError, WgetResult},
    http::common::Protocol,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ParsedUrl {
    pub protocol: Protocol,
    pub domain_name: String,
    pub port: u16,
    pub path: String,
    pub filename: String,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> WgetResult<Self> {
        let mut parts = url.split('/');
        let protocol_str = parts
            .next()
            .ok_or_else(|| WgetError::ParsingError("Empty url!".to_string()))?;
        let protocol = match protocol_str {
            "http:" => Protocol::Http,
            _ => {
                return Err(WgetError::ParsingError(format!(
                    "Unknown protocol {}",
                    protocol_str
                )))
            }
        };
        let _ = parts.next();
        let (domain_name, port) = {
            let domain_name_port = parts
                .next()
                .ok_or_else(|| {
                    WgetError::ParsingError("Invalid url, unable to read domain name!".to_string())
                })?
                .to_string();
            if let Some(loc) = domain_name_port.rfind(':') {
                let (domain_name, port_str) = domain_name_port.split_at(loc);
                let port_str = &port_str[1..]; // remove colon
                (
                    domain_name.to_string(),
                    port_str.parse::<u16>().map_err(|_| {
                        WgetError::ParsingError(format!("Invalid port {0}", port_str))
                    })?,
                )
            } else {
                (domain_name_port, protocol.get_port())
            }
        };
        let path = parts.filter(|p| !p.is_empty()).collect::<Vec<&str>>();
        let filename = path.last().unwrap_or(&"index.html").to_string();
        let path = path.join("/");
        let path = format!("/{}", path);
        Ok(ParsedUrl {
            protocol,
            domain_name,
            port,
            path,
            filename,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_url() {
        let url = ParsedUrl::parse("http://google.com").expect("Couldn't parse!");
        assert_eq!(
            url,
            ParsedUrl {
                protocol: Protocol::Http,
                domain_name: "google.com".to_string(),
                port: 80,
                path: "/".to_string(),
                filename: "index.html".to_string()
            }
        )
    }

    #[test]
    fn parses_uncommon_url() {
        let url = ParsedUrl::parse("http://test").expect("Couldn't parse!");
        assert_eq!(
            url,
            ParsedUrl {
                protocol: Protocol::Http,
                domain_name: "test".to_string(),
                port: 80,
                path: "/".to_string(),
                filename: "index.html".to_string()
            }
        )
    }

    #[test]
    fn parses_url_with_port() {
        let url = ParsedUrl::parse("http://test:8080").expect("Couldn't parse!");
        assert_eq!(
            url,
            ParsedUrl {
                protocol: Protocol::Http,
                domain_name: "test".to_string(),
                port: 8080,
                path: "/".to_string(),
                filename: "index.html".to_string()
            }
        )
    }

    #[test]
    fn parses_url_with_path() {
        let url = ParsedUrl::parse("http://test/my_site.html").expect("Couldn't parse!");
        assert_eq!(
            url,
            ParsedUrl {
                protocol: Protocol::Http,
                domain_name: "test".to_string(),
                port: 80,
                path: "/my_site.html".to_string(),
                filename: "my_site.html".to_string()
            }
        )
    }

    #[test]
    fn parses_url_with_port_and_path() {
        let url = ParsedUrl::parse("http://test:8080/my_site.html").expect("Couldn't parse!");
        assert_eq!(
            url,
            ParsedUrl {
                protocol: Protocol::Http,
                domain_name: "test".to_string(),
                port: 8080,
                path: "/my_site.html".to_string(),
                filename: "my_site.html".to_string()
            }
        )
    }
}
