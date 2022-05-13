use reqwest;
use serde_json::Value;
use log::debug;

mod session;
use session::Session;

const IGNORE_INVALID_CERT: bool = true; // TODO config

#[tokio::main]
async fn main() {

    let ses = Session::builder()
        .ignore_invalid_ssl_cert(IGNORE_INVALID_CERT)
        .http_client()
        .build()
        .unwrap();

    /*
    let obj = send_http().await.unwrap();
    println!("RES = {:?}", obj);
    */
}

// will take a sip msg
async fn send_http() -> Result<serde_json::Map<String, Value>, ()> {

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(IGNORE_INVALID_CERT)
        .build()
        .unwrap();

    let res = client.post("https://localhost/osrf-gateway-v1")
        .body("service=open-ils.auth&method=opensrf.system.echo&param=\"yo\"")
        .send()
        .await
        .unwrap();

    debug!("HTTP response status: {}", res.status());

    let obj = match res.json().await {
        Ok(v) => {
            match v {
                Value::Object(map) => map,
                _ => panic!("Uh Oh: {:?}", v)
            }
        },
        Err(e) => panic!("Also Un Oh: {:?}", e)
    };

    return Ok(obj);
}


