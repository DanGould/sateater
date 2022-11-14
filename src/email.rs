use lettre::Message;
use rusoto_ses::{RawMessage, SendRawEmailRequest, Ses, SesClient};

// uses ~/.aws/credentials
pub async fn send_email(subject: String, body: String) {
    let email = Message::builder()
        .from("nolooking Forward <913burner@gmail.com>".parse().unwrap())
        .to("nolooking Forward <913burner@gmail.com>".parse().unwrap())
        .subject(subject.clone())
        .body(body.clone())
        .unwrap();

    let raw_email = email.formatted();
    let ses_client = SesClient::new(rusoto_core::Region::UsEast1);
    let ses_request = SendRawEmailRequest {
        raw_message: RawMessage {
            data: base64::encode(raw_email).into(),
        },
        ..Default::default()
    };

    match ses_client.send_raw_email(ses_request).await {
        Ok(_) => {}
        Err(e) => eprintln!("FAILED TO SEND EMAIL:\n\n{}\n\n{}\n\n{}", subject, body, e),
    };
}
