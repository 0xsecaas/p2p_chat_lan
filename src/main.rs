use std::sync::Arc;
mod cli;
mod network;
mod peer;
mod signal;
mod walkie_talkie;

use clap::Parser;
use cli::*;
use walkie_talkie::WalkieTalkie;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Only handle CLI commands
    match cli.command {
        Commands::Start { port, name } => {
            let walkie_talkie = WalkieTalkie::new(name, port);
            let wt_arc = Arc::new(walkie_talkie);
            let wt_signal = wt_arc.clone();
            tokio::spawn(async move {
                crate::signal::handle_signals(wt_signal).await;
            });
            wt_arc.start().await?;
        }
    }

    Ok(())
}
