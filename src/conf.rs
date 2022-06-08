pub enum HttpProto {
    Http,
    Https
}

pub struct Config {
    pub sip_address: String,
    pub sip_port: u16,
    pub http_host: String,
    pub http_port: u16,
    pub http_proto: String,
    pub http_path: String,
    pub max_clients: usize,
    pub ascii: bool,
    pub daemonize: bool,
    pub ignore_ssl_errors: bool,
}

