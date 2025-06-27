use crate::network::handle_tcp_connection;
use crate::peer::{Message, NetworkMessage, PeerInfo};
use chrono::Utc;
use colored::*;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{broadcast, Mutex};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

pub struct WalkieTalkie {
    pub peer_id: String,
    pub name: String,
    pub port: u16,
    pub peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    pub message_sender: broadcast::Sender<String>,
}

impl WalkieTalkie {
    pub fn new(name: String, port: u16) -> Self {
        let peer_id = Uuid::new_v4().to_string();
        let (message_sender, _) = broadcast::channel(100);

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
        let tcp_listener = self.start_tcp_listener();
        let discovery_broadcaster = self.start_discovery_broadcast();
        let discovery_listener = self.start_discovery_listener();
        let heartbeat_sender = self.start_heartbeat();
        let cli_handler = self.start_cli_handler();
        let message_display = self.start_message_display();

        // Run all services
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

    async fn start_tcp_listener(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        println!(
            "🔗 TCP listener started on port {}",
            self.port.to_string().bright_blue()
        );

        loop {
            let (stream, addr) = listener.accept().await?;
            let peers = self.peers.clone();
            let message_sender = self.message_sender.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_tcp_connection(stream, addr, peers, message_sender).await {
                    eprintln!("Error handling TCP connection from {}: {}", addr, e);
                }
            });
        }
    }

    async fn start_discovery_broadcast(&self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;

        let local_ip = local_ip_address::local_ip()?;
        let peer_info = PeerInfo {
            id: self.peer_id.clone(),
            name: self.name.clone(),
            ip: local_ip,
            port: self.port,
        };

        loop {
            let discovery_msg = NetworkMessage::Discovery(peer_info.clone());
            let msg_bytes = serde_json::to_vec(&discovery_msg)?;

            // Broadcast on common subnet
            if let Err(e) = socket.send_to(&msg_bytes, "255.255.255.255:9999").await {
                eprintln!("Failed to send discovery broadcast: {}", e);
            }

            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn start_discovery_listener(&self) -> Result<(), Box<dyn std::error::Error>> {
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
                                // Don't add ourselves
                                if peer_info.id != self.peer_id {
                                    let mut peers = self.peers.lock().await;
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
                                // Keep peer alive
                                let peers = self.peers.lock().await;
                                if peers.contains_key(&peer_id) {
                                    // Peer is still alive, extend its lifetime
                                }
                            }
                            NetworkMessage::Exit(peer_id) => {
                                let mut peers = self.peers.lock().await;
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

    async fn start_heartbeat(&self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;

        loop {
            let heartbeat = NetworkMessage::Heartbeat(self.peer_id.clone());
            let msg_bytes = serde_json::to_vec(&heartbeat)?;

            if let Err(e) = socket.send_to(&msg_bytes, "255.255.255.255:9999").await {
                eprintln!("Failed to send heartbeat: {}", e);
            }

            sleep(Duration::from_secs(10)).await;
        }
    }

    async fn start_cli_handler(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📋 Commands:");
        println!("  /list    - List discovered peers");
        println!("  /msg <message> - Send message to all peers");
        println!("  /quit    - Quit the application");
        println!("  Just type any message to broadcast it!\n");

        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            print!("\u{1F4AC} ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            line.clear();
            if reader.read_line(&mut line).await? == 0 {
                break; // EOF
            }

            let input = line.trim();
            if input.is_empty() {
                continue;
            }

            match input {
                "/quit" => {
                    // Broadcast exit message to all peers before quitting
                    let exit_msg = NetworkMessage::Exit(self.peer_id.clone());
                    let msg_bytes = serde_json::to_vec(&exit_msg)?;
                    let peers = self.peers.lock().await;
                    for peer in peers.values() {
                        if let Ok(mut stream) = TcpStream::connect((peer.ip, peer.port)).await {
                            let _ = stream.write_all(&msg_bytes).await;
                            println!("Quit broadcasted to {} ({})", peer.name, peer.id);
                        }
                    }
                    println!("\u{1F44B} Now Goodbye!!");
                    std::process::exit(0);
                }
                "/list" => {
                    let peers = self.peers.lock().await;
                    if peers.is_empty() {
                        println!("📭 No peers discovered yet.");
                    } else {
                        println!("👥 Discovered peers:");
                        for peer in peers.values() {
                            println!(
                                "  - {} ({}) at {}:{}",
                                peer.name, peer.id, peer.ip, peer.port
                            );
                        }
                    }
                }
                _ => {
                    let message_content = if input.starts_with("/msg ") {
                        input.strip_prefix("/msg ").unwrap()
                    } else {
                        input
                    };

                    if let Err(e) = self.broadcast_message(message_content).await {
                        eprintln!("Failed to send message: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn start_message_display(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut receiver = self.message_sender.subscribe();

        loop {
            match receiver.recv().await {
                Ok(message) => {
                    println!("\n📨 {}", message);
                    print!("💬 ");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    eprintln!("Message display lagged, continuing...");
                }
            }
        }

        Ok(())
    }

    async fn broadcast_message(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let message = Message {
            from_id: self.peer_id.clone(),
            from_name: self.name.clone(),
            content: content.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };

        let network_msg = NetworkMessage::Chat(message.clone());
        let msg_bytes = serde_json::to_vec(&network_msg)?;

        let peers = self.peers.lock().await;
        let mut successful_sends = 0;

        for peer in peers.values() {
            match TcpStream::connect((peer.ip, peer.port)).await {
                Ok(mut stream) => {
                    if stream.write_all(&msg_bytes).await.is_ok() {
                        successful_sends += 1;
                    }
                }
                Err(_) => {
                    // Peer might be offline, skip
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
}
