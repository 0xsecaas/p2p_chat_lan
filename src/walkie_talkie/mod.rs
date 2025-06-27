//! WalkieTalkie module: Provides the main struct and logic for peer-to-peer walkie-talkie functionality, including peer management, message broadcasting, and coordination of submodules.
//!
//! This module defines the `WalkieTalkie` struct, which represents a peer in the walkie-talkie network.
//! It handles the initialization of the peer, starting of necessary services like TCP listener,
//! mDNS discovery, heartbeat sending, and CLI handling. It also provides functionality to broadcast
//! messages to other peers.

pub mod net {
    pub mod broadcast;
    pub mod discovery;
    pub mod heartbeat;
    pub mod listener;
}

pub mod display {
    pub mod cli;
    pub mod message_display;
}

use crate::error::WalkieTalkieError;
use crate::peer::PeerInfo;
use colored::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone)]
pub struct WalkieTalkie {
    pub peer_id: String,
    pub name: String,
    pub port: u16,
    pub peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    pub message_sender: tokio::sync::broadcast::Sender<String>,
}

impl WalkieTalkie {
    pub fn new(name: String, port: u16) -> Self {
        // Validate name and port
        let valid_name = name.trim();
        let name = if valid_name.is_empty() || valid_name.len() > 64 {
            "Anonymous".to_string()
        } else {
            valid_name.to_string()
        };
        let port = if port == 0 { 8080 } else { port };
        let peer_id = Uuid::new_v4().to_string();
        let (message_sender, _) = tokio::sync::broadcast::channel(100);
        Self {
            peer_id,
            name,
            port,
            peers: Arc::new(Mutex::new(HashMap::new())),
            message_sender,
        }
    }
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "{}",
            "🎙️  Starting P2P Walkie-Talkie...".bright_cyan().bold()
        );
        println!("👤 Your ID: {}", self.peer_id.bright_yellow());
        println!("📡 Your Name: {}", self.name.bright_green());
        println!(
            "🔌 Listening on port: {}",
            self.port.to_string().bright_blue()
        );

        // Start all services concurrently
        let tcp_listener = net::listener::start_tcp_listener(self);
        let mdns_discovery = net::discovery::start_mdns(Arc::new(self.clone()));
        let heartbeat_sender = net::heartbeat::start_heartbeat(self);
        let cli_handler = display::cli::start_cli_handler(self);
        let message_display = display::message_display::start_message_display(self);

        tokio::select! {
            result = tcp_listener => {
                if let Err(e) = result {
                    eprintln!("TCP listener error: {}", e);
                    std::process::exit(1);
                }
            }
            result = mdns_discovery => {
                if let Err(e) = result {
                    eprintln!("mDNS discovery error: {}", e);
                    std::process::exit(1);
                }
            }
            result = heartbeat_sender => {
                if let Err(e) = result {
                    eprintln!("Heartbeat sender error: {}", e);
                    std::process::exit(1);
                }
            }
            result = cli_handler => {
                if let Err(e) = result {
                    eprintln!("CLI handler error: {}", e);
                    std::process::exit(1);
                }
            }
            result = message_display => {
                if let Err(e) = result {
                    eprintln!("Message display error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Ok(())
    }
    pub async fn broadcast_message(&self, content: &str) -> Result<(), WalkieTalkieError> {
        net::broadcast::broadcast_message(self, content).await
    }
}
