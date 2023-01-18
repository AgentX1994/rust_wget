use std::collections::HashMap;

use crate::{connection::Connection, error::WgetResult, url::ParsedUrl, Configuration};

#[derive(Debug, Default)]
pub struct ConnectionCache {
    connections: HashMap<(String, u16), Connection>,
}

impl ConnectionCache {
    pub fn get_connection(
        &mut self,
        url: &ParsedUrl,
        config: &Configuration,
    ) -> WgetResult<&mut Connection> {
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
                let conn = Connection::new(url.domain_name.to_string(), url.port, config)?;
                Ok(v.insert(conn))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        hint,
        net::TcpListener,
        os::unix::prelude::AsRawFd,
        sync::{
            atomic::{AtomicU16, Ordering},
            Arc,
        },
        thread::{self, JoinHandle},
    };

    use crate::protocol::Protocol;

    use super::*;

    fn get_listener_port(listener: &TcpListener) -> u16 {
        listener
            .local_addr()
            .expect("Listener has no local addr!")
            .port()
    }

    fn create_listener_thread() -> (u16, JoinHandle<()>) {
        let port_atomic = Arc::new(AtomicU16::new(0));
        let port_atomic_t = port_atomic.clone();
        let t = thread::spawn(move || {
            let listener = TcpListener::bind("localhost:0").expect("Could not create listener");
            let port = get_listener_port(&listener);

            port_atomic_t.store(port, Ordering::Relaxed);

            for conn in listener.incoming() {
                let _ = conn.expect("Error in incoming");
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
    fn can_create_default() {
        let conn_cache = ConnectionCache::default();
        assert_eq!(conn_cache.connections.len(), 0);
    }

    #[test]
    fn creates_connection() {
        let mut conn_cache = ConnectionCache::default();
        let config = Configuration { debug: 0 };
        let (port, _t) = create_listener_thread();

        let url = ParsedUrl {
            protocol: Protocol::Http,
            domain_name: "localhost".to_string(),
            port,
            path: "/".to_string(),
            filename: "index.html".to_string(),
        };
        let _conn = conn_cache
            .get_connection(&url, &config)
            .expect("Could not connect!");
    }

    #[test]
    fn reuses_connection() {
        let mut conn_cache = ConnectionCache::default();
        let config = Configuration { debug: 0 };
        let (port, _t) = create_listener_thread();

        // The only way I can think of to check if two TcpStream objects are the
        // same are to check the fds

        let fd1 = {
            let url = ParsedUrl {
                protocol: Protocol::Http,
                domain_name: "localhost".to_string(),
                port,
                path: "/".to_string(),
                filename: "index.html".to_string(),
            };
            let conn = conn_cache
                .get_connection(&url, &config)
                .expect("Could not connect!");
            #[cfg(not(target_os = "windows"))]
            {
                conn.get_socket().as_raw_fd()
            }
            #[cfg(target_os = "windows")]
            {
                conn.as_raw_socket()
            }
        };

        let fd2 = {
            let url = ParsedUrl {
                protocol: Protocol::Http,
                domain_name: "localhost".to_string(),
                port,
                path: "/test.html".to_string(),
                filename: "test.html".to_string(),
            };
            let conn = conn_cache
                .get_connection(&url, &config)
                .expect("Could not connect!");
            #[cfg(not(target_os = "windows"))]
            {
                conn.get_socket().as_raw_fd()
            }
            #[cfg(target_os = "windows")]
            {
                conn.as_raw_socket()
            }
        };
        assert_eq!(fd1, fd2);
    }

    #[test]
    fn creates_new_connection() {
        let mut conn_cache = ConnectionCache::default();
        let config = Configuration { debug: 0 };
        let (port1, _t1) = create_listener_thread();
        let (port2, _t2) = create_listener_thread();

        // The only way I can think of to check if two TcpStream objects are the
        // same are to check the fds

        let fd1 = {
            let url = ParsedUrl {
                protocol: Protocol::Http,
                domain_name: "localhost".to_string(),
                port: port1,
                path: "/".to_string(),
                filename: "index.html".to_string(),
            };
            let conn = conn_cache
                .get_connection(&url, &config)
                .expect("Could not connect!");
            #[cfg(not(target_os = "windows"))]
            {
                conn.get_socket().as_raw_fd()
            }
            #[cfg(target_os = "windows")]
            {
                conn.as_raw_socket()
            }
        };

        let fd2 = {
            let url = ParsedUrl {
                protocol: Protocol::Http,
                domain_name: "localhost".to_string(),
                port: port2,
                path: "/".to_string(),
                filename: "index.html".to_string(),
            };
            let conn = conn_cache
                .get_connection(&url, &config)
                .expect("Could not connect!");
            #[cfg(not(target_os = "windows"))]
            {
                conn.get_socket().as_raw_fd()
            }
            #[cfg(target_os = "windows")]
            {
                conn.as_raw_socket()
            }
        };
        assert_ne!(fd1, fd2);
    }
}
