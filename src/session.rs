use log::{debug, error};
use reqwest;
use serde_urlencoded as urlencoded;
use sip2;
use std::fmt;
use uuid::Uuid;

pub struct Session {
    /// Unique session identifier
    key: String,

    /// E.g. https://localhost/sip2-mediator
    http_url: String,

    client: reqwest::blocking::Client,
    // sip socket
}

impl Session {
    pub fn builder() -> SessionBuilder {
        SessionBuilder::new()
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
            .client
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

pub struct SessionBuilder {
    key: String,
    http_url: Option<String>,
    client: Option<reqwest::blocking::Client>,
    // sip socket,
    ignore_invalid_ssl_cert: bool,
}

impl SessionBuilder {
    pub fn new() -> SessionBuilder {
        let key = Uuid::new_v4().as_simple().to_string()[0..16].to_string();

        SessionBuilder {
            key,
            client: None,
            http_url: None,
            ignore_invalid_ssl_cert: false,
        }
    }

    pub fn ignore_invalid_ssl_cert(&mut self, value: bool) -> &mut SessionBuilder {
        self.ignore_invalid_ssl_cert = value;
        self
    }

    pub fn http_url(&mut self, http_url: &str) -> &mut SessionBuilder {
        self.http_url = Some(http_url.to_string());
        self
    }

    pub fn http_client(&mut self) -> &mut SessionBuilder {
        let builder = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(self.ignore_invalid_ssl_cert);

        match builder.build() {
            Ok(c) => self.client = Some(c),
            Err(e) => error!("{} Error building HTTP client: {}", self, e),
        }

        self
    }

    pub fn build(&self) -> Result<Session, String> {
        let client = match &self.client {
            Some(c) => c,
            None => {
                return Err(format!(
                    "{} Attempt to create a Session without an HTTP client",
                    self
                ));
            }
        };

        let http_url = match &self.http_url {
            Some(h) => h,
            None => {
                return Err(format!(
                    "{} Attempt to create a Session without an HTTP http_url",
                    self
                ));
            }
        };

        Ok(Session {
            key: self.key.to_owned(),
            http_url: http_url.to_owned(),
            client: client.to_owned(),
        })
    }
}

impl fmt::Display for SessionBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Session {}", self.key)
    }
}
