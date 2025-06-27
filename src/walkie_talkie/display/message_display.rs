use crate::walkie_talkie::WalkieTalkie;
use tokio::sync::broadcast;

pub async fn start_message_display(wt: &WalkieTalkie) -> Result<(), Box<dyn std::error::Error>> {
    let mut receiver = wt.message_sender.subscribe();
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
