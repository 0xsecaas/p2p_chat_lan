mod cli;
mod network;
mod peer;
mod walkie_talkie;

use clap::Parser;
use cli::*;
use walkie_talkie::WalkieTalkie;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { port, name } => {
            let walkie_talkie = WalkieTalkie::new(name, port);
            walkie_talkie.start().await?;
        }
    }

    Ok(())
}
