use std::{path::Path, time::SystemTime};

use rocket::{
    http::Status,
    serde::json::{serde_json, Json},
};
use serde::{Deserialize, Serialize};
use std::io::Write;

// fn load_api_key() -> String {
//     std::fs::read_to_string("VOLTAGE_API_SECRET")
//         .expect("could not open voltage_api_secret")
//         .trim()
//         .to_string()
// }

// Send email with request (and refund address)
// Store on machine also ( no db yet )
// Reply with address

// If paid, we will open a channel for you in next few hours

// Fix capacity at 1m sats
// Fix at 1 month duration (until we can get quotes from API).

// let mut map = HashMap::new();
// map.insert("product_id", "61909b26da0e257a68863f25");
// map.insert("remote_balance", &capacity);
// map.insert("local_balance", "0");
// map.insert("channel_expiry", &duration);

// let client = reqwest::Client::new();
// let res = client
//     .post(url)
//     .header("X-VOLTAGE-AUTH", load_api_key().as_bytes())
//     .json(&map)
//     .send()
//     .await
//     .expect("contacted voltage");

// let txt = res.text().await.expect("loads");
// let json = serde_json::from_str(&txt).unwrap();
// dbg!(&json);

// Write to file just in case something goes wrong with email
fn store_request(data: String) {
    let logfile = "inbound.log";
    if !Path::new(logfile).exists() {
        std::fs::File::create(logfile).expect("created inbound.log");
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(logfile)
        .unwrap();

    if let Err(e) = writeln!(
        file,
        "{:?}: {}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        data,
    ) {
        panic!("Couldn't write to file: {}", e);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InboundRequest {
    nodeid: String,
    capacity: u32,
    duration: u32,
    refund_address: String,
    payment_address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InboundResponse {
    price: u32,
    size: u32,
    duration: u32,
    address: String,
}

#[get("/getinbound?<nodeid>&<capacity>&<duration>&<refund_address>")]
pub async fn getinbound(
    nodeid: String,
    capacity: u32,
    duration: u32,
    refund_address: String,
) -> (Status, Json<InboundResponse>) {
    let price = 30_000;
    let resp_capacity = 1_000_000; //sats
    let resp_duration = 1; //month
    let payment_address = crate::lnd::get_onchain_address().await;

    let subject = format!("[Channel Request]: {}", nodeid);
    let body = format!(
        "
[Channel Request]
-------------------
NodeID: {}
Capacity: {}
Duration: {}
Payment Address: https://mempool.space/address/{}
Refund Address: {}
-------------------
Cost: {}
        ",
        nodeid, capacity, duration, payment_address, refund_address, price
    );

    let channel_request = InboundRequest {
        nodeid,
        capacity,
        duration,
        refund_address,
        payment_address: payment_address.clone(),
    };

    let request_str = serde_json::to_string(&channel_request).unwrap();
    store_request(request_str);
    crate::email::send_email(subject, body).await;

    (
        Status::Accepted,
        Json(InboundResponse {
            price,
            size: resp_capacity,
            duration: resp_duration,
            address: payment_address,
        }),
    )
}
