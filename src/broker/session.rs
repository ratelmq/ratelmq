use std::net::IpAddr;

pub struct Session {
    client_id: String,
    ip: IpAddr,
    persistent: bool,
}

impl Session {
    pub fn new(client_id: String, ip: IpAddr, persistent: bool) -> Self {
        Session {
            client_id,
            ip,
            persistent,
        }
    }
}
