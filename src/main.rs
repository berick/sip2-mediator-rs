use getopts;
use log::{info, LevelFilter, SetLoggerError};
use std::env;
use syslog::{BasicLogger, Facility, Formatter3164};

pub mod conf;
pub mod server;
pub mod session;

const HELP_TEXT: &str = r#"

Options:

    --sip-address <sip-address>
        Listen address for SIP server.

    --sip-port <sip-port>
        List port for SIP server.

    --http-host <http-host>
        Hostname of HTTP API server.

    --http-port <http-port>
        Port for HTTP API server.

    --http-proto <http-proto>
        Protocoal for HTTP API server. http or https.

    --http-path <http-path>
        URL path for HTTP API server

    --max-clients <max-clients>
        Maximum number of SIP client connections allowed.

    --syslog-facility <syslog-facility>

    --syslog-level <syslog-level>

    --ascii
        Normalize and encode data returned to SIP clients as ASCII.
        Otherwise, uses UTF8.

    --daemonize
        Detach and background the process.
"#;

fn main() {
    let conf = parse_args();

    setup_logging(&conf);

    let mut server = server::Server::new(conf);
    server.serve();
}

fn setup_logging(config: &conf::Config) {
    let formatter = Formatter3164 {
        facility: Facility::LOG_LOCAL4, // TODO from config
        hostname: None,
        process: "sip2-mediator".into(),
        pid: std::process::id(),
    };

    let logger = match syslog::unix(formatter) {
        Ok(logger) => logger,
        Err(e) => {
            eprintln!("Cannot connect to syslog: {:?}", e);
            return;
        }
    };

    // TODO loglevel from config
    log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .expect("Boxed logger setup with loglevel");
}

fn parse_args() -> conf::Config {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();

    opts.optopt("", "sip-address", "", "");
    opts.optopt("", "sip-port", "", "");
    opts.optopt("", "http-host", "", "");
    opts.optopt("", "http-port", "", "");
    opts.optopt("", "http-proto", "", "");
    opts.optopt("", "http-path", "", "");
    opts.optopt("", "max-clients", "", "");
    opts.optopt("", "syslog-facility", "", "");
    opts.optopt("", "ascii", "", "");
    opts.optopt("", "daemonize", "", "");
    opts.optopt("", "ignore-ssl-errors", "", "");
    opts.optopt("", "help", "", "");

    let options = opts
        .parse(&args[1..])
        .expect("Error parsing command line options");

    if options.opt_present("help") {
        println!("{}", HELP_TEXT);
        std::process::exit(0);
    }

    // Shorthand for extracting option values
    let opstr = |v, d| options.opt_str(v).unwrap_or(String::from(d));

    let sip_port_str = opstr("sip-port", "6001");
    let http_port_str = opstr("http-port", "80");
    let max_clients_str = opstr("max-clients", "256");

    let sip_port = sip_port_str.parse::<u16>().expect("Invalid SIP port");
    let http_port = http_port_str.parse::<u16>().expect("Invalid HTTP port");
    let max_clients = max_clients_str
        .parse::<usize>()
        .expect("Invalid Max Clients");

    conf::Config {
        sip_address: opstr("sip-address", "localhost"),
        sip_port,
        http_host: opstr("http-host", "localhost"),
        http_port,
        http_proto: opstr("http-proto", "http"),
        http_path: opstr("http-path", "/sip2-mediator"),
        max_clients,
        ascii: options.opt_present("ascii"),
        daemonize: options.opt_present("daemonize"),
        ignore_ssl_errors: options.opt_present("ignore-ssl-errors"),
    }
}
