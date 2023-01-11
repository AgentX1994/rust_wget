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
    pub fn parse(mut url: &str, config: &Configuration) -> WgetResult<Self> {
        // First, look for the protocol by looking for a ':' character
        // If none found, assume HTTP
        let protocol = if let Some(colon_index) = url.find(':') {
            let (protocol_str, rest) = url.split_at(colon_index);
            let protocol = protocol_str.parse()?;
            url = &rest[1..]; // remove ':'
            protocol
        } else {
            if config.debug > 0 {
                println!("No protocol found, assuming HTTP!");
            }
            Protocol::Http
        };
        // If we find a // skip it
        if &url[..2] == "//" {
            url = &url[2..];
        };
        // Split at / to split the domain name + maybe port section from the path
        let (domain_and_port_str, path) = if let Some(slash_index) = url.find('/') {
            url.split_at(slash_index)
        } else {
            (url, "/")
        };
        // Determine if we are looking at a domain.name:port pair, or just a domain name
        // TODO: IPv6 domains
        let (domain_name, port) = {
            if let Some(loc) = domain_and_port_str.rfind(':') {
                let (domain_name, port_str) = domain_and_port_str.split_at(loc);
                let port_str = &port_str[1..]; // remove colon
                (
                    domain_name.to_string(),
                    port_str.parse::<u16>().map_err(|_| {
                        WgetError::ParsingError(format!("Invalid port {0}", port_str))
                    })?,
                )
            } else {
                (domain_and_port_str.to_string(), protocol.get_port())
            }
        };
        let path = path.to_string();
        // Grab the final file name from the path, or default to "index.html"
        let filename = if let Some(loc) = path.rfind('/') {
            if loc + 1 >= path.len() {
                "index.html".to_string()
            } else {
                path[loc + 1..].to_string()
            }
        } else {
            "index.html".to_string()
        };

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
