//! CLI display module: Handles user input, command parsing, and broadcasting messages or exit signals to peers.
//!
//! This module provides the functionality for the command-line interface (CLI) of the application,
//! allowing users to interact with the Walkie Talkie network. It handles user commands such as
//! listing peers, sending messages, and quitting the application. Additionally, it manages the
//! broadcasting of exit signals to all connected peers when a user decides to quit.

use crate::error::WalkieTalkieError;
use crate::peer::NetworkMessage;
use crate::walkie_talkie::WalkieTalkie;
use serde_json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub async fn broadcast_exit(wt: &WalkieTalkie) -> Result<(), WalkieTalkieError> {
    let exit_msg = NetworkMessage::Exit(wt.peer_id.clone());
    let msg_bytes = serde_json::to_vec(&exit_msg)?;
    let peers = wt.peers.lock().await;
    for peer in peers.values() {
        if let Ok(mut stream) = TcpStream::connect((peer.ip, peer.port)).await {
            let _ = stream.write_all(&msg_bytes).await;
            println!("Quit broadcasted to {} ({})", peer.name, peer.id);
        }
    }
    Ok(())
}

pub async fn start_cli_handler(wt: &WalkieTalkie) -> Result<(), WalkieTalkieError> {
    println!("\n📋 Commands:");
    println!("  /list    - List discovered peers");
    println!("  /msg <message> - Send message to all peers");
    println!("  /quit    - Quit the application");
    println!("  Just type any message to broadcast it!\n");

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        print!("💬 ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break;
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        // Validate input length
        if input.len() > 512 {
            println!("Input too long. Please keep messages under 512 characters.");
            continue;
        }
        match input {
            "/quit" => {
                if let Err(e) = crate::walkie_talkie::display::cli::broadcast_exit(wt).await {
                    eprintln!("Error broadcasting exit: {}", e);
                }
                println!("\u{1F44B} Now Goodbye!");
                std::process::exit(0);
            }
            "/list" => {
                let peers = wt.peers.lock().await;
                if peers.is_empty() {
                    println!("📭 No peers discovered yet.");
                } else {
                    println!("👥 Discovered peers:");
                    for peer in peers.values() {
                        if !peer.is_valid() {
                            println!("  - Invalid peer: {:?}", peer);
                            continue;
                        }
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
                if let Err(e) = wt.broadcast_message(message_content).await {
                    eprintln!("Failed to send message: {}", e);
                }
            }
        }
    }
    Ok(())
}
