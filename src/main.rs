use reqwest;
use serde_json::Value;
use log::debug;

mod session;
use session::Session;

const IGNORE_INVALID_CERT: bool = true; // TODO config

//#[tokio::main]
fn main() {

    let ses = Session::builder()
        .ignore_invalid_ssl_cert(IGNORE_INVALID_CERT)
        .http_url("https://localhost/osrf-gateway-v1")
        .http_client()
        .build()
        .unwrap();

    /*
    let obj = send_http().await.unwrap();
    println!("RES = {:?}", obj);
    */
}

