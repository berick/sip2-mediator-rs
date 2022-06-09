use super::conf::Config;
use super::session::Session;
use log::{error, info};
use sip2;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

/*
    let ses = Session::builder()
        .ignore_invalid_ssl_cert(IGNORE_INVALID_CERT)
        .http_url("https://localhost/osrf-gateway-v1")
        .http_client()
        .build()
        .expect("Failed to build SIP session");
*/

pub struct Server {
    config: Config,
    connections: usize,
}

impl Server {
    pub fn new(config: Config) -> Server {
        Server {
            config,
            connections: 0,
        }
    }

    pub fn serve(&mut self) {
        let bind = format!("{}:{}", self.config.sip_address, self.config.sip_port);

        let listener = TcpListener::bind(bind).expect("Error starting SIP server");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    //thread::spawn(|| self.handle_connection(stream));
                }
                Err(e) => {
                    error!("Error accepting TCP connection {}", e);
                }
            }

            // TODO
            // check max connections and do some thread yielding as needed.
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        match stream.peer_addr() {
            Ok(a) => info!("New SIP connection from {}", a),
            Err(e) => {
                error!("SIP connection has no peer addr? {}", e);
                return;
            }
        }
    }
}
