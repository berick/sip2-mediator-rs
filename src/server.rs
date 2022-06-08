use sip2;
use super::session::Session;
use super::conf::Config;


/*
    let ses = Session::builder()
        .ignore_invalid_ssl_cert(IGNORE_INVALID_CERT)
        .http_url("https://localhost/osrf-gateway-v1")
        .http_client()
        .build()
        .expect("Failed to build SIP session");
*/


pub struct Server {
    conf: Config,
}


impl Server {

    pub fn new(conf: Config) -> Server {
        Server {
            conf,
        }
    }

    pub fn serve(&mut self) {
    }
}

