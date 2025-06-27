use crate::error::WalkieTalkieError;
use crate::peer::{Message, NetworkMessage};
use crate::walkie_talkie::WalkieTalkie;
use serde_json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn broadcast_message(wt: &WalkieTalkie, content: &str) -> Result<(), WalkieTalkieError> {
    let message = Message {
        from_id: wt.peer_id.clone(),
        from_name: wt.name.clone(),
        content: content.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| WalkieTalkieError::Unknown(e.to_string()))?
            .as_secs(),
    };
    let network_msg = NetworkMessage::Chat(message.clone());
    let msg_bytes = serde_json::to_vec(&network_msg)?;
    let peers = wt.peers.lock().await;
    let mut successful_sends = 0;
    for peer in peers.values() {
        if !peer.is_valid() {
            eprintln!("Skipping invalid peer: {:?}", peer);
            continue;
        }
        if let Ok(mut stream) = TcpStream::connect((peer.ip, peer.port)).await {
            if stream.write_all(&msg_bytes).await.is_ok() {
                successful_sends += 1;
            }
        }
    }
    if successful_sends > 0 {
        println!("📤 Message sent to {} peer(s)", successful_sends);
    } else {
        println!("📭 No peers available to receive the message");
    }
    Ok(())
}
