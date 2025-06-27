mod cli;
mod gui;
mod network;
mod peer;
mod walkie_talkie;

use clap::Parser;
use cli::*;
use gui::run_gui;
use walkie_talkie::WalkieTalkie;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // If no command is given, launch the GUI
    if std::env::args().len() == 2 {
        run_gui();
        return Ok(());
    }

    match cli.command {
        Commands::Start { port, name } => {
            let walkie_talkie = WalkieTalkie::new(name, port);
            walkie_talkie.start().await?;
        }
    }

    Ok(())
}
