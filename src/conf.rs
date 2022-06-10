#[derive(Debug, Clone)]
pub struct Config {
    pub sip_address: String,
    pub sip_port: u16,
    pub http_host: String,
    pub http_port: u16,
    pub http_proto: String,
    pub http_path: String,
    pub syslog_facility: String,
    pub syslog_level: String,
    pub max_clients: usize,
    pub ascii: bool,
    pub daemonize: bool,
    pub ignore_ssl_errors: bool,
}
