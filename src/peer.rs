//! Peer module: Defines peer information, message types, and network message enums for the P2P walkie-talkie.
//!
//! This module contains the structures and enums used for peer discovery, messaging, and network
//! communication in the P2P walkie-talkie application. It includes the `PeerInfo` struct for
//! identifying peers in the network, the `Message` struct for chat messages, and the `NetworkMessage`
//! enum for different types of network messages.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from_id: String,
    pub from_name: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Discovery(PeerInfo),
    Chat(Message),
    Heartbeat(String), // peer_id
    Exit(String),      // peer_id
}
