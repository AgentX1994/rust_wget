pub mod connection_cache;
pub mod error;
pub mod http;
pub mod protocol;
pub mod url;

#[derive(Debug, Default)]
pub struct Configuration {
    pub debug: u8,
}
