use crate::peer::{NetworkMessage, PeerInfo};
use crate::walkie_talkie::WalkieTalkie;
use chrono::Utc;
use colored::*;
use local_ip_address::local_ip;
use serde_json;
use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};

pub async fn start_discovery_broadcast(
    wt: &WalkieTalkie,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;

    let local_ip = local_ip()?;
    let peer_info = PeerInfo {
        id: wt.peer_id.clone(),
        name: wt.name.clone(),
        ip: local_ip,
        port: wt.port,
    };

    loop {
        let discovery_msg = NetworkMessage::Discovery(peer_info.clone());
        let msg_bytes = serde_json::to_vec(&discovery_msg)?;

        if let Err(e) = socket.send_to(&msg_bytes, "255.255.255.255:9999").await {
            eprintln!("Failed to send discovery broadcast: {}", e);
        }
        sleep(Duration::from_secs(5)).await;
    }
}

pub async fn start_discovery_listener(wt: &WalkieTalkie) -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:9999").await?;
    println!(
        "👂 Discovery listener started on port {}",
        "9999".bright_blue()
    );
    let mut buf = [0; 1024];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, _addr)) => {
                if let Ok(network_msg) = serde_json::from_slice::<NetworkMessage>(&buf[..len]) {
                    match network_msg {
                        NetworkMessage::Discovery(peer_info) => {
                            if peer_info.id != wt.peer_id {
                                let mut peers = wt.peers.lock().await;
                                if !peers.contains_key(&peer_info.id) {
                                    let timestamp = Utc::now().format("%H:%M:%S");
                                    println!(
                                        "[{}] {} Discovered new peer: {} ({})",
                                        timestamp.to_string().dimmed(),
                                        "🔍".bright_green(),
                                        peer_info.name.bright_cyan(),
                                        peer_info.ip.to_string().yellow()
                                    );
                                }
                                peers.insert(peer_info.id.clone(), peer_info);
                            }
                        }
                        NetworkMessage::Heartbeat(peer_id) => {
                            let peers = wt.peers.lock().await;
                            if peers.contains_key(&peer_id) {
                                // Peer is still alive
                            }
                        }
                        NetworkMessage::Exit(peer_id) => {
                            let mut peers = wt.peers.lock().await;
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
            Err(e) => {
                eprintln!("Error receiving discovery message: {}", e);
            }
        }
    }
}
