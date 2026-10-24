//! What the daemon's tests share.

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// A webhook on this machine that takes every request with `status`, and the bodies it got.
pub async fn receiver(status: &'static str) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("binds");
    let address = format!(
        "http://{}/hooks/secret-path",
        listener.local_addr().expect("address")
    );
    let bodies = Arc::new(Mutex::new(Vec::new()));
    let got = Arc::clone(&bodies);
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let mut request = Vec::new();
            let mut buffer = [0_u8; 4096];
            // Reads until the body promised in the headers has come.
            loop {
                let read = stream.read(&mut buffer).await.unwrap_or(0);
                request.extend_from_slice(&buffer[..read]);
                let text = String::from_utf8_lossy(&request).to_string();
                if let Some((head, body)) = text.split_once("\r\n\r\n") {
                    let length = head
                        .lines()
                        .find_map(|line| {
                            line.to_ascii_lowercase()
                                .strip_prefix("content-length: ")
                                .map(str::to_owned)
                        })
                        .and_then(|length| length.trim().parse::<usize>().ok())
                        .unwrap_or(0);
                    if body.len() >= length {
                        got.lock().expect("lock").push(body.to_owned());
                        break;
                    }
                }
                if read == 0 {
                    break;
                }
            }
            let _ = stream
                .write_all(format!("HTTP/1.1 {status}\r\ncontent-length: 0\r\n\r\n").as_bytes())
                .await;
        }
    });
    (address, bodies)
}

/// A status page on this machine that answers every request with `body`, and its address.
pub async fn status_page(body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("binds");
    let address = format!("http://{}", listener.local_addr().expect("address"));
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let mut buffer = [0_u8; 4096];
            let _ = stream.read(&mut buffer).await;
            let _ = stream
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: \
                         {}\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await;
        }
    });
    address
}
