use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use crate::model::ApiResponse;
use tracing::{debug, error, info};

pub fn spawn_fetcher(
    url: String,
    refresh: Duration,
    insecure: bool,
    api_key: Option<String>,
    api_key_header: Option<String>,
    tx: Sender<Result<ApiResponse, String>>,
) {
    thread::spawn(move || {
        info!("fetcher started");
        let client = match reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(insecure)
            .timeout(Duration::from_secs(5))
            .build()
        {
            Ok(client) => client,
            Err(err) => {
                error!("client error: {err}");
                let _ = tx.send(Err(format!("Client error: {err}")));
                return;
            }
        };

        loop {
            let result = fetch_once(&client, &url, api_key.as_deref(), api_key_header.as_deref());
            if tx.send(result).is_err() {
                debug!("receiver dropped, exiting fetcher");
                break;
            }
            if refresh > Duration::from_secs(0) {
                thread::sleep(refresh);
            }
        }
    });
}

fn fetch_once(
    client: &reqwest::blocking::Client,
    url: &str,
    api_key: Option<&str>,
    api_key_header: Option<&str>,
) -> Result<ApiResponse, String> {
    let mut req = client.get(url);
    if let (Some(key), Some(header)) = (api_key, api_key_header) {
        if !key.trim().is_empty() && !header.trim().is_empty() {
            req = req.header(header, key);
        }
    }
    let resp = req.send().map_err(|err| err.to_string())?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }
    resp.json::<ApiResponse>().map_err(|err| err.to_string())
}

#[cfg(all(test, feature = "net-tests"))]
mod tests {
    use super::fetch_once;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn fetch_once_paths() {
        let client = reqwest::blocking::Client::builder().build().unwrap();
        if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
            let addr = listener.local_addr().unwrap();

            thread::spawn(move || {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buf = [0u8; 512];
                    let _ = stream.read(&mut buf);
                    let body = r#"{"now":1,"messages":2,"aircraft":[]}"#;
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
            });

            let url = format!("http://{}", addr);
            let result = fetch_once(&client, &url, None, None).unwrap();
            assert_eq!(result.now, Some(1));
            assert_eq!(result.messages, Some(2));
            assert!(result.aircraft.is_empty());
        } else {
            let url = "http://127.0.0.1:1";
            let result = fetch_once(&client, url, None, None);
            assert!(result.is_err());
        }
    }
}
