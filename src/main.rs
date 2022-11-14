#[macro_use]
extern crate rocket;

pub mod cashu;
pub mod email;
pub mod inbound;
pub mod lnd;
pub mod wildcard;
use std::sync::Mutex;

use qrcode_generator::QrCodeEcc;
use rocket::{fs::FileServer, http::Status, serde::json::Json};
use sateater::{
    cashu::cashu_receive,
    vault::{board, land, BattleConfig},
    BOARD_SIZE,
};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct PaymentResponse {
    pub amount: i64,
    pub address: String,
    pub payment_id: String,
    pub message: String,
}

#[get("/create_payment?<amount>&<message>")]
pub async fn create_payment(
    amount: i64,
    message: Option<String>,
) -> (Status, Json<PaymentResponse>) {
    let (_, _, _, label) = lnd::load_conf();
    let description = match message {
        Some(message) => message,
        None => label,
    };

    let created_invoice = lnd::create_invoice(amount, description).await;
    let payment_id = hex::encode(created_invoice.r_hash);
    create_qr_code(
        created_invoice.payment_request.clone(),
        payment_id.to_string(),
    );

    (
        Status::Accepted,
        Json(PaymentResponse {
            amount,
            address: created_invoice.payment_request,
            payment_id: payment_id.to_string(),
            message: "payment created".to_string(),
        }),
    )
}

fn create_qr_code(qr_string: String, payment_id: String) {
    qrcode_generator::to_png_to_file(
        qr_string,
        QrCodeEcc::Low,
        512,
        format!("html/qr_codes/{}.png", payment_id),
    )
    .expect("created qr code OK")
}

#[derive(Serialize, Debug)]
pub struct PaymentStatusResponse {
    payment_complete: bool,
    confirmed_paid: u64,
    unconfirmed_paid: u64,
}

#[get("/check_payment?<payment_id>")]
pub async fn check_payment(payment_id: String) -> (Status, Json<PaymentStatusResponse>) {
    let response = PaymentStatusResponse {
        payment_complete: lnd::check_invoice(payment_id).await,
        // For later doing onchain
        confirmed_paid: 0,
        unconfirmed_paid: 0,
    };
    (Status::Accepted, Json(response))
}

#[get("/receive_ecash?<token>")]
pub async fn receive_ecash(token: String) -> (Status, Json<PaymentStatusResponse>) {
    let amount_paid = cashu_receive(&token);
    let response = PaymentStatusResponse {
        payment_complete: amount_paid > 0,
        // For later doing onchain
        confirmed_paid: amount_paid as u64,
        unconfirmed_paid: 0,
    };
    (Status::Accepted, Json(response))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from("./html"))
        .mount(
            "/api",
            routes![
                create_payment,
                check_payment,
                receive_ecash,
                wildcard::wildcard,
                board,
                land,
                inbound::getinbound
            ],
        )
        .manage(Mutex::new(BattleConfig {
            board: vec![vec![(0, 0); BOARD_SIZE.into()]; BOARD_SIZE.into()],
        }))
}
