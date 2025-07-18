//! Peer discovery module: Handles mDNS-based peer discovery and advertisement for the P2P Chat network.
//!
//! This module is responsible for discovering and advertising peers in the Chat network using mDNS.
//! It handles both the sending and receiving of peer information, as well as the management of discovered peers.

use crate::chat::Peer;
use crate::error::ChatError;
use crate::peer::{NetworkMessage, PeerInfo};
use futures_util::{pin_mut, stream::StreamExt};
use libmdns;
use mdns::{Record, RecordKind};
use std::{net::IpAddr, sync::Arc, time::Duration};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

const SERVICE_NAME: &str = "_chat._udp.local";

pub async fn start_mdns(peer: Arc<Peer>) -> Result<(), ChatError> {
    // Spawn advertisement in a blocking thread
    let peer_ad = peer.clone();
    tokio::task::spawn_blocking(move || {
        let responder = libmdns::Responder::new().unwrap();
        let _svc = responder.register(
            "_chat._udp".to_owned(),
            format!("{}-{}", peer_ad.name, peer_ad.peer_id),
            peer_ad.port,
            &[&format!("peer_id={}", peer_ad.peer_id)],
        );
        loop {
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    });

    // Discovery
    let stream = mdns::discover::all(SERVICE_NAME, Duration::from_secs(15))
        .map_err(|e| ChatError::Network(e.to_string()))?
        .listen();
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
            // fallback to our port if not found
            .unwrap_or(peer.port);
        // Validate peer_id and port
        if peer_id.is_empty() || peer_port == 0 {
            eprint!("⚠️  Warning: Discovered peer has invalid ID or port.");
            continue; // Skip invalid peer
        }
        // Validate peer_name (non-empty, reasonable length)
        if peer_name.trim().is_empty() || peer_name.len() > 128 {
            eprint!("⚠️  Warning: Discovered peer has invalid name.");
            continue; // Skip invalid peer name
        }
        if let Some(ip) = addr {
            // Ignore self
            if peer_id == peer.peer_id {
                continue;
            }
            // Validate IP address (skip loopback and multicast)
            if ip.is_loopback() || ip.is_multicast() {
                eprint!("⚠️  Warning: Discovered peer has invalid IP address.");
                continue;
            }
            let peer_info = PeerInfo {
                id: peer_id.clone(),
                name: peer_name.clone(),
                ip,
                port: peer_port, // Use discovered port
            };
            if !peer_info.is_valid() {
                eprint!(
                    "⚠️  Warning: Discovered peer has invalid PeerInfo. {:?}",
                    peer_info
                );
                continue;
            }
            let mut peers = peer.peers.lock().await;
            if !peers.contains_key(&peer_info.id) {
                println!(
                    "🔍 Discovered peer via mDNS: {} at {}:{}",
                    peer_name, ip, peer_port
                );
                // Try to send our PeerInfo to the new peer via TCP
                let my_info = PeerInfo {
                    id: peer.peer_id.clone(),
                    name: peer.name.clone(),
                    ip, // fallback to discovered IP if local IP is not available
                    port: peer.port,
                };
                if !my_info.is_valid() {
                    println!(
                        "⚠️  Warning: Our PeerInfo is invalid, not sending discovery message."
                    );
                    continue;
                }
                let msg = NetworkMessage::Discovery(my_info);
                let socket_addr = std::net::SocketAddr::new(ip, peer_port);
                let msg_bytes = serde_json::to_vec(&msg).unwrap();
                tokio::spawn(async move {
                    if let Ok(mut stream) = TcpStream::connect(socket_addr).await {
                        let _ = stream.write_all(&msg_bytes).await;
                    }
                });
            }
            peers.insert(peer_info.id.clone(), peer_info);
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
