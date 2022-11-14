use configparser::ini::Ini;
use tonic_lnd::lnrpc::AddInvoiceResponse;

pub fn load_conf() -> (String, String, String, String) {
    let mut config = Ini::new();
    let _map = config
        .load("./config.cfg")
        .expect("config.cfg does not exist! please copy config_example.cfg");

    let address = config.get("lnd", "address").expect("address provided");
    let cert = config.get("lnd", "certfile").expect("cert provided");
    let macaroon = config
        .get("lnd", "macaroonfile")
        .expect("macaroon provided");
    let label = config
        .get("lnd", "defaultlabel")
        .expect("default label provided");
    (address, cert, macaroon, label)
}

pub async fn get_onchain_address() -> String {
    let (address, cert, macaroon, _) = load_conf();
    let mut client = tonic_lnd::connect_lightning(address, cert, macaroon)
        .await
        .expect("failed to connect");

    let newaddressreq = tonic_lnd::lnrpc::NewAddressRequest {
        r#type: 4, //taproot
        ..Default::default()
    };

    let address = client
        .new_address(newaddressreq)
        .await
        .expect(
            "created invoice with lightning node using ssh tunnel!\n
                 Run `ssh pi@your.node.ip -q -N -L 10009:localhost:10009`\n\n",
            // Run `ssh -nNT -L ./lightning-rpc:/root/.lightning/bitcoin/lightning-rpc user@host`\n\n"
        )
        .into_inner();

    address.address.to_string()
}

pub async fn create_invoice(amount: i64, description: String) -> AddInvoiceResponse {
    let (address, cert, macaroon, _) = load_conf();
    let mut client = tonic_lnd::connect_lightning(address, cert, macaroon)
        .await
        .expect("failed to connect");

    // let sat_amount = amount.checked_mul(1000).expect("not billions(?) of sats") as i64;

    let invoice = tonic_lnd::lnrpc::Invoice {
        memo: description,
        value: amount,
        ..Default::default()
    };

    let created_invoice = client
        .add_invoice(invoice)
        .await
        // , &payment_id.to_string(), &description, None)
        .expect(
            "created invoice with lightning node using ssh tunnel!\n
             Run `ssh pi@your.node.ip -q -N -L 10009:localhost:10009`\n\n",
            // Run `ssh -nNT -L ./lightning-rpc:/root/.lightning/bitcoin/lightning-rpc user@host`\n\n"
        )
        .into_inner();

    created_invoice
}

pub async fn check_invoice(payment_id: String) -> bool {
    let (address, cert, macaroon, _label) = load_conf();
    let mut client = tonic_lnd::connect_lightning(address, cert, macaroon)
        .await
        .expect("failed to connect");

    let payment_hash = tonic_lnd::lnrpc::PaymentHash {
        r_hash: hex::decode(payment_id).expect("valid payment hash"),
        ..Default::default()
    };

    let invoice = client
        .lookup_invoice(payment_hash)
        .await
        .expect("fetched invoices")
        .into_inner();

    let payment_complete = if invoice.state == 1 { true } else { false };
    payment_complete
}
