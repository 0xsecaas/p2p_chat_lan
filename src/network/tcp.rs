use crate::peer::{NetworkMessage, PeerInfo};
use chrono::Utc;
use colored::*;
use serde_json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex};

pub async fn handle_tcp_connection(
    stream: TcpStream,
    _addr: SocketAddr,
    peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    message_sender: broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0; 1024];

    while let Ok(_n) = stream.readable().await {
        match stream.try_read(&mut buf) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                if let Ok(network_msg) = serde_json::from_slice::<NetworkMessage>(&buf[..n]) {
                    match network_msg {
                        NetworkMessage::Chat(message) => {
                            let display_msg =
                                format!("{} says: {}", message.from_name, message.content);
                            let _ = message_sender.send(display_msg);
                        }
                        NetworkMessage::Exit(peer_id) => {
                            let mut peers = peers.lock().await;
                            if peers.remove(&peer_id).is_some() {
                                let timestamp = Utc::now().format("%H:%M:%S");
                                println!(
                                    "[{}] {} Peer {} exited and was removed from the list.",
                                    timestamp.to_string().dimmed(),
                                    "❌".bright_red(),
                                    peer_id.bright_yellow()
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                eprintln!("Error reading from TCP stream: {}", e);
                break;
            }
        }
    }

    Ok(())
}
