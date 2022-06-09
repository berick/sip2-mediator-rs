use log::{trace, debug, info, error};
use reqwest;
use serde_urlencoded as urlencoded;
use std::net;
use std::io;
use sip2;
use std::fmt;
use uuid::Uuid;
use super::conf;

/// Manages the connection between a SIP client and the HTTP backend.
pub struct Session {

    config: conf::Config,

    sip_connection: sip2::Connection,

    /// Unique session identifier
    key: String,

    /// E.g. https://localhost/sip2-mediator
    http_url: String,

    http_client: reqwest::blocking::Client,
}

impl Session {

    /// Our thread starts here.  If anything fails, we just log it and
    /// go away so as not to disrupt the main server thread.
    pub fn run(config: conf::Config, stream: net::TcpStream) {

        match stream.peer_addr() {
            Ok(a) => info!("New SIP connection from {}", a),
            Err(e) => {
                error!("SIP connection has no peer addr? {}", e);
                return;
            }
        }

        let key = Uuid::new_v4().as_simple().to_string()[0..16].to_string();

        let http_url = format!("{}://{}:{}/{}",
            &config.http_proto,
            &config.http_host,
            config.http_port,
            &config.http_path
        );

        let http_builder = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(config.ignore_ssl_errors);

        let http_client = match http_builder.build() {
            Ok(c) => c,
            Err(e) => {
                error!("Error building HTTP client: {}; exiting", e);
                return;
            }
        };

        let mut ses = Session {
            config,
            key,
            http_url,
            http_client,
            sip_connection: sip2::Connection::new_from_stream(stream)
        };

        ses.start();
    }

    fn start(&mut self) {
        debug!("Session starting");

        loop {

            let sip_msg = match self.sip_connection.recv() {
                Ok(sm) => sm,
                Err(e) => {
                    error!("SIP receive failed; session exiting: {}", e);
                    return;
                }
            };

            trace!("Read SIP message: {}", sip_msg);
        }
    }

    fn http_round_trip(self, msg: sip2::Message) -> Result<sip2::Message, ()> {
        let msg_json = match msg.to_json() {
            Ok(m) => m,
            Err(e) => {
                error!("Failed translating SIP message to JSON: {}", e);
                return Err(());
            }
        };

        let msg_encoded = match urlencoded::to_string(msg_json) {
            Ok(m) => m,
            Err(e) => {
                error!("Error url-encoding SIP message: {}", e);
                return Err(());
            }
        };

        let key_encoded = match urlencoded::to_string(&self.key) {
            Ok(k) => k,
            Err(e) => {
                error!("Error url-encoding session key: {}", e);
                return Err(());
            }
        };

        let request = self
            .http_client
            .post(&self.http_url)
            .header(reqwest::header::CONNECTION, "keep-alive")
            .body(format!("session={}&message={}", key_encoded, msg_encoded));

        let res = match request.send() {
            Ok(v) => v,
            Err(e) => {
                error!("{} HTTP request failed : {}", self, e);
                return Err(());
            }
        };

        if res.status() != 200 {
            error!(
                "HTTP server responded with a non-200 status: status={} res={:?}",
                res.status(),
                res
            );
            return Err(());
        }

        debug!("HTTP response status: {} {}", res.status(), self);

        let msg_json: String = match res.text() {
            Ok(v) => v,
            Err(e) => {
                error!("{} HTTP response failed to ready body text: {}", self, e);
                return Err(());
            }
        };

        debug!("{} HTTP response JSON: {}", self, msg_json);

        match sip2::Message::from_json(&msg_json) {
            Ok(m) => Ok(m),
            Err(e) => {
                error!("http_round_trip from_json error: {}", e);
                return Err(());
            }
        }
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Session {}", self.key)
    }
}

