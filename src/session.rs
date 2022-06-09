use super::conf;
use log::{debug, error, info, trace};
use reqwest;
use serde_urlencoded as urlencoded;
use sip2;
use std::fmt;
use std::io;
use std::net;
use uuid::Uuid;

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

        let http_url = format!(
            "{}://{}:{}{}",
            &config.http_proto, &config.http_host, config.http_port, &config.http_path
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
            sip_connection: sip2::Connection::new_from_stream(stream),
        };

        ses.start();
    }

    fn start(&mut self) {
        debug!("Session starting");

        loop {
            // TODO send end-session message when needed

            // Blocks waiting for a SIP request to arrive
            let sip_req = match self.sip_connection.recv() {
                Ok(sm) => sm,
                Err(e) => {
                    error!("SIP receive failed; session exiting: {}", e);
                    break;
                }
            };

            trace!("Read SIP message: {}", sip_req);

            let sip_resp = match self.http_round_trip(sip_req) {
                Ok(r) => r,
                _ => {
                    error!("Error processing SIP request. Session exiting");
                    break;
                }
            };

            if let Err(e) = self.sip_connection.send(&sip_resp) {
                error!(
                    "{} Error relaying response back to SIP client: {}. shutting down session",
                    self, e
                );
                break;
            }

            debug!("{} Successfully relayed response back to SIP client", self);
        }

        info!("SIP session {} shutting down", self);

        self.sip_connection.disconnect().ok();
    }

    /// Send a SIP client request to the HTTP backend for processing.
    ///
    /// Blocks waiting for a response.
    fn http_round_trip(&self, msg: sip2::Message) -> Result<sip2::Message, ()> {
        let msg_json = match msg.to_json() {
            Ok(m) => m,
            Err(e) => {
                error!("Failed translating SIP message to JSON: {}", e);
                return Err(());
            }
        };

        let values = [("session", &self.key), ("message", &msg_json)];

        let body = match urlencoded::to_string(&values) {
            Ok(m) => m,
            Err(e) => {
                error!("Error url-encoding SIP message: {}", e);
                return Err(());
            }
        };

        trace!("POST content: {}", body);

        let request = self
            .http_client
            .post(&self.http_url)
            .header(reqwest::header::CONNECTION, "keep-alive")
            .body(body);

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
