use reqwest;
use serde_json::Value;

const IGNORE_INVALID_CERT: bool = true; // TODO config

#[tokio::main]
async fn main() {

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(IGNORE_INVALID_CERT)
        .build()
        .unwrap();

    /*
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(IGNORE_INVALID_CERT)
        .build()
        .unwrap();

    let post = client.post("https://localhost/osrf-gateway-v1")
        .body("service=open-ils.auth&method=opensrf.system.echo&param=\"yo\"");

    let res = post.send().unwrap();

    let obj = match res.json() {
        Ok(v) => {
            match v {
                Value::Object(map) => map,
                _ => panic!("Uh Oh: {:?}", v)
            }
        },
        Err(e) => panic!("Also Un Oh: {:?}", e)
    };
    */

    let res = client.post("https://localhost/osrf-gateway-v1")
        .body("service=open-ils.auth&method=opensrf.system.echo&param=\"yo\"")
        .send()
        .await
        .unwrap();

    println!("STAT = {}", res.status());

    let obj = match res.json().await {
        Ok(v) => {
            match v {
                Value::Object(map) => map,
                _ => panic!("Uh Oh: {:?}", v)
            }
        },
        Err(e) => panic!("Also Un Oh: {:?}", e)
    };

    println!("RES = {:?}", obj);
}


