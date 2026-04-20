use std::io::Write;

use croc_sidecar::{Croc, CrocEvent};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> croc_sidecar::Result {
    print!("Insert the receive code\n> ");
    std::io::stdout().flush()?;

    let mut code = String::new();
    std::io::stdin().read_line(&mut code)?;

    let mut out = std::env::current_dir()?;
    out.push("examples/receive_files");
    let mut croc = Croc::new().receive().out(out).spawn(code)?;

    let mut events = croc.events()?;
    while let Some(event) = events.next().await {
        match event {
            CrocEvent::ReceivingInfo(info) => {
                println!("Receiving {} ({} bytes)", info.name, info.size);
            }
            CrocEvent::ReceivingFrom(relay) => {
                println!("Receiving file from {}", relay);
            }
            CrocEvent::Receiving(progress) => {
                let mut str = format!("Receiving {}: {}%", progress.file_name, progress.percentage);
                if let Some(speed) = progress.speed {
                    str.push_str(&format!(" ({} bytes/s)", speed));
                }
                println!("{}", str);
            }
            CrocEvent::Done => {
                println!("Transfer done.");
            }
            CrocEvent::EOF => {
                println!("Failed to complete transference");
            }
            _ => {}
        }
    }

    Ok(())
}
