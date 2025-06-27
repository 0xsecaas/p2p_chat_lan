use crate::peer::{NetworkMessage, PeerInfo};
use crate::walkie_talkie::WalkieTalkie;
use futures_util::{pin_mut, stream::StreamExt};
use libmdns;
use mdns::{Record, RecordKind};
use std::{net::IpAddr, sync::Arc, time::Duration};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

const SERVICE_NAME: &str = "_walkietalkie._udp.local";

pub async fn start_mdns(wt: Arc<WalkieTalkie>) -> Result<(), Box<dyn std::error::Error>> {
    // Spawn advertisement in a blocking thread
    let wt_ad = wt.clone();
    tokio::task::spawn_blocking(move || {
        let responder = libmdns::Responder::new().unwrap();
        let _svc = responder.register(
            "_walkietalkie._udp".to_owned(),
            format!("{}-{}", wt_ad.name, wt_ad.peer_id),
            wt_ad.port,
            &[&format!("peer_id={}", wt_ad.peer_id)],
        );
        loop {
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    });

    // Discovery
    let stream = mdns::discover::all(SERVICE_NAME, Duration::from_secs(15))?.listen();
    pin_mut!(stream);
    while let Some(Ok(response)) = stream.next().await {
        let addr = response.records().filter_map(to_ip_addr).next();
        let peer_name = response
            .records()
            .find_map(|r| match &r.kind {
                RecordKind::PTR(name) => Some(name.to_string()),
                _ => None,
            })
            .unwrap_or_else(|| "unknown".to_string());
        let peer_id = response
            .records()
            .find_map(|r| match &r.kind {
                RecordKind::TXT(ref txts) => {
                    for txt in txts {
                        if let Some(id) = txt.strip_prefix("peer_id=") {
                            return Some(id.to_string());
                        }
                    }
                    None
                }
                _ => None,
            })
            .unwrap_or_else(|| peer_name.clone());
        // Extract port from SRV record if available
        let peer_port = response
            .records()
            .find_map(|r| match &r.kind {
                RecordKind::SRV { port, .. } => Some(*port),
                _ => None,
            })
            .unwrap_or(wt.port); // fallback to our port if not found
        if let Some(ip) = addr {
            // Ignore self
            if peer_id == wt.peer_id {
                continue;
            }
            let peer = PeerInfo {
                id: peer_id.clone(),
                name: peer_name.clone(),
                ip,
                port: peer_port, // Use discovered port
            };
            let mut peers = wt.peers.lock().await;
            if !peers.contains_key(&peer.id) {
                println!(
                    "🔍 Discovered peer via mDNS: {} at {}:{}",
                    peer_name, ip, peer_port
                );
                // Try to send our PeerInfo to the new peer via TCP
                let my_info = PeerInfo {
                    id: wt.peer_id.clone(),
                    name: wt.name.clone(),
                    ip, // fallback to discovered IP if local IP is not available
                    port: wt.port,
                };
                let msg = NetworkMessage::Discovery(my_info);
                let socket_addr = std::net::SocketAddr::new(ip, peer_port);
                let msg_bytes = serde_json::to_vec(&msg).unwrap();
                tokio::spawn(async move {
                    if let Ok(mut stream) = TcpStream::connect(socket_addr).await {
                        let _ = stream.write_all(&msg_bytes).await;
                    }
                });
            }
            peers.insert(peer.id.clone(), peer);
        }
    }
    Ok(())
}

fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}
