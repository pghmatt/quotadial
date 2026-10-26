//! What the daemon's tests share.

use std::path::Path;
use std::sync::{Arc, Mutex};

use quotadial_core::licence::TEST_SEED;
use quotadial_core::{Licence, Plan};

use crate::{DataDir, Licensing, licence};

/// The licensing of a data folder at `dir` that keeps a QuotaDial Pro key, signed with the test key
/// a debug build trusts.
pub fn pro(dir: &Path) -> Licensing {
    let data = DataDir::new(dir.to_path_buf());
    let key = Licence {
        id: "test-pro".into(),
        plan: Plan::Pro,
        issued: "2026-10-24".parse().expect("a date"),
        until: None,
    }
    .sign(&TEST_SEED);
    licence::add(&data, &key, quotadial_core::licence::today()).expect("adds the key");
    Licensing::new(Some(&data))
}

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
            // It takes one request on each connection and then closes it, so it says so: told
            // nothing, a client keeps the connection for its next request, which can then go to a
            // connection already closed and never arrive, as a retry did on main's CI.
            let _ = stream
                .write_all(
                    format!("HTTP/1.1 {status}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n")
                        .as_bytes(),
                )
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
            // One request on each connection, as `receiver` takes, and said in the same way.
            let _ = stream
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: \
                         {}\r\nconnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await;
        }
    });
    address
}

#[cfg(test)]
mod tests {
    use super::{receiver, status_page};

    /// What a stand-in's answer says about its connection.
    fn connection(answer: &reqwest::Response) -> Option<&str> {
        answer
            .headers()
            .get("connection")
            .and_then(|value| value.to_str().ok())
    }

    /// Each stand-in takes one request on a connection and then closes it, so its answer says so,
    /// and a client that keeps connections opens a new one for its next request, which arrives.
    /// Told nothing, the client can send it down the closed one, where it is lost: a webhook's retry
    /// on main's CI, 2026-10-26.
    #[tokio::test]
    async fn the_stand_ins_say_they_close_each_connection() {
        let client = reqwest::Client::new();
        let (address, bodies) = receiver("404 Not Found").await;
        for _ in 0..2 {
            let answer = client
                .post(&address)
                .body("{}")
                .send()
                .await
                .expect("answers");
            assert_eq!(connection(&answer), Some("close"));
        }
        assert_eq!(bodies.lock().expect("lock").len(), 2, "both arrived");
        let page = status_page("{}").await;
        let answer = client.get(&page).send().await.expect("answers");
        assert_eq!(connection(&answer), Some("close"));
    }
}
