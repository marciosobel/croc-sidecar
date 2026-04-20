use croc_sidecar::{Croc, CrocEvent};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> croc_sidecar::Result {
    let mut file = std::env::current_dir()?;
    file.push("examples/send_files/test.txt");

    let mut croc = Croc::new().yes(false).send().file(file).spawn()?;
    let mut events = croc.events()?;

    while let Some(event) = events.next().await {
        match event {
            CrocEvent::Hashing(progress) => {
                println!("Hashing {}: {}%", progress.file_name, progress.percentage);
            }
            CrocEvent::SendingInfo(info) => {
                println!("Sending {} ({} bytes)", info.name, info.size);
            }
            CrocEvent::CodeGenerated(code) => {
                println!("Code generated: {}", code);
            }
            CrocEvent::SendingTo(relay) => {
                println!("Sending file to {}", relay);
            }
            CrocEvent::Sending(progress) => {
                let mut str = format!("Sending {}: {}%", progress.file_name, progress.percentage);
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
