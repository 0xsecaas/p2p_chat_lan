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

use crate::peer::PeerInfo;
use colored::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct WalkieTalkie {
    pub peer_id: String,
    pub name: String,
    pub port: u16,
    pub peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    pub message_sender: tokio::sync::broadcast::Sender<String>,
}

impl WalkieTalkie {
    pub fn new(name: String, port: u16) -> Self {
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
        let discovery_broadcaster = net::discovery::start_discovery_broadcast(self);
        let discovery_listener = net::discovery::start_discovery_listener(self);
        let heartbeat_sender = net::heartbeat::start_heartbeat(self);
        let cli_handler = display::cli::start_cli_handler(self);
        let message_display = display::message_display::start_message_display(self);

        tokio::select! {
            result = tcp_listener => {
                if let Err(e) = result {
                    eprintln!("TCP listener error: {}", e);
                }
            }
            result = discovery_broadcaster => {
                if let Err(e) = result {
                    eprintln!("Discovery broadcaster error: {}", e);
                }
            }
            result = discovery_listener => {
                if let Err(e) = result {
                    eprintln!("Discovery listener error: {}", e);
                }
            }
            result = heartbeat_sender => {
                if let Err(e) = result {
                    eprintln!("Heartbeat sender error: {}", e);
                }
            }
            result = cli_handler => {
                if let Err(e) = result {
                    eprintln!("CLI handler error: {}", e);
                }
            }
            result = message_display => {
                if let Err(e) = result {
                    eprintln!("Message display error: {}", e);
                }
            }
        }
        Ok(())
    }
    pub async fn broadcast_message(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        net::broadcast::broadcast_message(self, content).await
    }
}
