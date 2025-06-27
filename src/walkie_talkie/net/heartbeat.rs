use crate::peer::NetworkMessage;
use crate::walkie_talkie::WalkieTalkie;
use serde_json;
use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};

pub async fn start_heartbeat(wt: &WalkieTalkie) -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;
    loop {
        let heartbeat = NetworkMessage::Heartbeat(wt.peer_id.clone());
        let msg_bytes = serde_json::to_vec(&heartbeat)?;
        if let Err(e) = socket.send_to(&msg_bytes, "255.255.255.255:9999").await {
            eprintln!("Failed to send heartbeat: {}", e);
        }
        sleep(Duration::from_secs(10)).await;
    }
}
