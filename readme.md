# Croc Sidecar
Wrap the [`croc`] binary into idiomatic structs and integrate them within your apps easily.

## Usage

### Sending a file

```rust
use croc_sidecar::Croc;

fn main() -> croc_sidecar::Result {
    let croc = Croc::new()
        .send()
        .code("my-custom-code")
        .file("/path/to/file.rs")
        .spawn()?;
    Ok(())
}
```

### Receiving a file

```rust
use croc_sidecar::Croc;

fn main() -> croc_sidecar::Result {
    let croc = Croc::new()
        .receive()
        .spawn("my-custom-code")?;
    Ok(())
}
```

### Listening to events

You can capture progress, hashing status, and other events by consuming the asynchronous event stream:

```rust
use croc_sidecar::Croc;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> croc_sidecar::Result {
    let mut croc = Croc::new().send().file("file.rs").spawn()?;

    let mut stream = croc.events()?;
    while let Some(event) = stream.next().await {
        match event {
            croc_sidecar::CrocEvent::CodeGenerated(code) => println!("Code is: {code}"),
            croc_sidecar::CrocEvent::Hashing(progress) => println!("Hashing: {}%", progress.percentage),
            croc_sidecar::CrocEvent::Sending(progress) => println!("Sending: {}%", progress.percentage),
            _ => {}
        }
    }
    Ok(())
}
```

---

This project does not own or maintain [`croc`]. Support the original project [here](https://github.com/sponsors/schollz).

[`croc`]: https://github.com/schollz/croc
