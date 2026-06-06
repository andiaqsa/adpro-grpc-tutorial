pub mod services {
    tonic::include_proto!("services");
}

use services::{payment_service_client::PaymentServiceClient, PaymentRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PaymentServiceClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(PaymentRequest {
        user_id: "andi_aqsa_123".into(),
        amount: 100.0,
    });

    let response = client.process_payment(request).await?;

    println!("RESPONSE DARI SERVER: {:?}", response.get_ref().success);

    Ok(())
}