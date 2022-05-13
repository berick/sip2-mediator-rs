use std::fmt;
use log::{debug, error};
use serde_json::Value;
use uuid::Uuid;
use reqwest;
use sip2;

pub struct SessionBuilder {
    key: String,
    client: Option<reqwest::Client>,
    // sip socket,
    ignore_invalid_ssl_cert: bool,
}

impl SessionBuilder {

    pub fn new() -> SessionBuilder {

        let key = Uuid::new_v4().as_simple().to_string()[0..16].to_string();

        SessionBuilder {
            key,
            client: None,
            ignore_invalid_ssl_cert: false
        }
    }

    pub fn ignore_invalid_ssl_cert(&mut self, value: bool) -> &mut SessionBuilder {
        self.ignore_invalid_ssl_cert = value;
        self
    }

    pub fn http_client(&mut self) -> &mut SessionBuilder {

        let builder = reqwest::Client::builder()
            .danger_accept_invalid_certs(self.ignore_invalid_ssl_cert);

        match builder.build() {
            Ok(c) => self.client = Some(c),
            Err(e) => error!("{} Error building HTTP client: {}", self.key, e)
        }

        self
    }

    pub fn build(&self) -> Result<Session, String> {

        let client = match &self.client {
            Some(c) => c,
            None => {
                return Err(format!("{} Attempt to create a Session without an HTTP client", self.key));
            }
        };

        Ok(Session { 
            key: self.key.to_owned(), 
            client: client.to_owned() 
        })
    }
}

pub struct Session {
    key: String,
    client: reqwest::Client,
    // socket
}

impl Session {

    pub fn builder() -> SessionBuilder {
        SessionBuilder::new()
    }

    async fn http_round_trip(self, msg: sip2::Message) -> Option<sip2::Message> {

        let request = self.client.post("https://localhost/osrf-gateway-v1")
            .body("service=open-ils.auth&method=opensrf.system.echo&param=\"yo\"");

        let res = match request.send().await {
            Ok(v) => v,
            Err(e) => {
                error!("{} HTTP request failed : {}", self, e);
                return None;
            }
        };

        debug!("HTTP response status: {} {}", res.status(), self);

        let msg_json: String = match res.text().await {
            Ok(v) => v,
            Err(e) => {
                error!("{} HTTP response failed to ready body text: {}", self, e);
                return None;
            }
        };

        debug!("{} HTTP response JSON: {}", self, msg_json);

        // TODO json-to-sip

        None
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {                 
        write!(f, "Session {}", self.key)
    }
}


