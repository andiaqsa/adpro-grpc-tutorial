use tonic::transport::Channel;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::io::{self, AsyncBufReadExt};

pub mod services {
    tonic::include_proto!("services");
}

use services::{
    payment_service_client::PaymentServiceClient, PaymentRequest,
    transaction_service_client::TransactionServiceClient, TransactionRequest,
    chat_service_client::ChatServiceClient, ChatMessage,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Eksekusi Payment Service (Unary)
    println!("--- 1. Testing Unary (Payment) ---");
    let mut payment_client = PaymentServiceClient::connect("http://[::1]:50051").await?;
    let req_payment = tonic::Request::new(PaymentRequest {
        user_id: "user_123".to_string(),
        amount: 100.0,
    });
    let res_payment = payment_client.process_payment(req_payment).await?;
    println!("RESPONSE={:?}", res_payment.into_inner());

    // 2. Eksekusi Transaction Service (Server Streaming)
    println!("\n--- 2. Testing Server Streaming (Transaction) ---");
    let mut transaction_client = TransactionServiceClient::connect("http://[::1]:50051").await?;
    let req_transaction = tonic::Request::new(TransactionRequest {
        user_id: "user_123".to_string(),
    });
    let mut stream = transaction_client.get_transaction_history(req_transaction).await?.into_inner();
    while let Some(transaction) = stream.message().await? {
        println!("Transaction: {:?}", transaction);
    }

    // 3. Eksekusi Chat Service (Bi-Directional Streaming)
    println!("\n--- 3. Testing Bi-Directional Streaming (Chat) ---");
    let channel = Channel::from_static("http://[::1]:50051").connect().await?;
    let mut chat_client = ChatServiceClient::new(channel);

    let (tx, rx): (Sender<ChatMessage>, Receiver<ChatMessage>) = mpsc::channel(32);

    // Background Task untuk mengirim pesan (Membaca input dari Terminal)
    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();

        println!("Ketik pesan Anda untuk CS (Ketik 'quit' untuk keluar):");
        while let Ok(Some(line)) = reader.next_line().await {
            if line.trim() == "quit" {
                break;
            }
            let msg = ChatMessage {
                user_id: "user_123".to_string(),
                message: line,
            };
            if tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    let outbound_stream = ReceiverStream::new(rx);
    let response = chat_client.chat(outbound_stream).await?;
    let mut inbound_stream = response.into_inner();

    // Menerima pesan balasan dari Server CS
    while let Some(message) = inbound_stream.message().await? {
        println!(">>> CS Virtual: {}", message.message);
    }

    Ok(())
}