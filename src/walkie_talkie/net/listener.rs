use crate::network::tcp::handle_tcp_connection;
use crate::walkie_talkie::WalkieTalkie;
use colored::*;
use tokio::net::TcpListener;

pub async fn start_tcp_listener(wt: &WalkieTalkie) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", wt.port)).await?;
    println!(
        "🔗 TCP listener started on port {}",
        wt.port.to_string().bright_blue()
    );

    loop {
        let (stream, addr) = listener.accept().await?;
        let peers = wt.peers.clone();
        let message_sender = wt.message_sender.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_tcp_connection(stream, addr, peers, message_sender).await {
                eprintln!("Error handling TCP connection from {}: {}", addr, e);
            }
        });
    }
}
