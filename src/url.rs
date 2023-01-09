use crate::{
    error::{WgetError, WgetResult},
    protocol::Protocol,
};

use super::Configuration;

#[derive(Debug, PartialEq, Eq)]
pub struct ParsedUrl {
    pub protocol: Protocol,
    pub domain_name: String,
    pub port: u16,
    pub path: String,
    pub filename: String,
}

impl ParsedUrl {
    pub fn parse(url: &str, config: &Configuration) -> WgetResult<Self> {
        let mut parts = url.split('/').peekable();
        let protocol_str = *parts
            .peek()
            .ok_or_else(|| WgetError::ParsingError("Empty url!".to_string()))?;
        let protocol = match protocol_str.parse::<Protocol>() {
            Ok(p) => {
                // Skip the protocol and empty section between the two // now
                let _ = parts.next();
                let _ = parts.next();
                p
            }
            Err(_) => {
                if config.debug > 0 {
                    println!("No protocol found, assuming HTTP!");
                }
                Protocol::Http
            } // Assume this is the domain name then
        };
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
        let config = Configuration { debug: 0 };
        let url = ParsedUrl::parse("http://google.com", &config).expect("Couldn't parse!");
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
        let config = Configuration { debug: 0 };
        let url = ParsedUrl::parse("http://test", &config).expect("Couldn't parse!");
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
    fn parses_url_without_protocol() {
        let config = Configuration { debug: 0 };
        let url = ParsedUrl::parse("www.google.com", &config).expect("Couldn't parse!");
        assert_eq!(
            url,
            ParsedUrl {
                protocol: Protocol::Http,
                domain_name: "www.google.com".to_string(),
                port: 80,
                path: "/".to_string(),
                filename: "index.html".to_string()
            }
        )
    }

    #[test]
    fn parses_url_with_port() {
        let config = Configuration { debug: 0 };
        let url = ParsedUrl::parse("http://test:8080", &config).expect("Couldn't parse!");
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
        let config = Configuration { debug: 0 };
        let url = ParsedUrl::parse("http://test/my_site.html", &config).expect("Couldn't parse!");
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
    fn parses_url_with_protocol() {
        let config = Configuration { debug: 0 };
        {
            let url = ParsedUrl::parse("http://test", &config).expect("Couldn't parse");
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
        {
            let url = ParsedUrl::parse("https://test", &config).expect("Couldn't parse");
            assert_eq!(
                url,
                ParsedUrl {
                    protocol: Protocol::Https,
                    domain_name: "test".to_string(),
                    port: 443,
                    path: "/".to_string(),
                    filename: "index.html".to_string()
                }
            )
        }
        {
            let url = ParsedUrl::parse("ftp://test", &config).expect("Couldn't parse");
            assert_eq!(
                url,
                ParsedUrl {
                    protocol: Protocol::Ftp,
                    domain_name: "test".to_string(),
                    port: 21,
                    path: "/".to_string(),
                    filename: "index.html".to_string()
                }
            )
        }
    }

    #[test]
    fn parses_url_with_port_and_path() {
        let config = Configuration { debug: 0 };
        let url =
            ParsedUrl::parse("http://test:8080/my_site.html", &config).expect("Couldn't parse!");
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
