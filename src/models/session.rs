use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::SystemTime;

pub struct Session {
    pub server: String,
    pub viewers: HashSet<String>,
    pub start_time: SystemTime,
    pub server_socket_addr: SocketAddr,
    pub name: String,
    pub os: String,
    pub version: String,
    pub control: bool,
}

impl Session {
    pub fn new(
        server: String,
        server_socket_addr: SocketAddr,
        name: String,
        os: String,
        version: String,
        control: bool,
    ) -> Self {
        Session {
            server,
            viewers: Default::default(),
            start_time: SystemTime::now(),
            server_socket_addr,
            name,
            os,
            version,
            control,
        }
    }
}
