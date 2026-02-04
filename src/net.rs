use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use crate::model::ApiResponse;
use tracing::{debug, error, info};

#[derive(Clone, Debug)]
struct SourceState {
    url: String,
    attempts: u32,
    backoff_until: Option<Instant>,
}

pub fn spawn_fetcher(
    urls: Vec<String>,
    refresh: Duration,
    insecure: bool,
    api_key: Option<String>,
    api_key_header: Option<String>,
    tx: Sender<Result<ApiResponse, String>>,
) {
    thread::spawn(move || {
        info!("fetcher started");
        let mut sources: Vec<SourceState> = urls
            .into_iter()
            .map(|u| u.trim().to_string())
            .filter(|u| !u.is_empty())
            .map(|url| SourceState {
                url,
                attempts: 0,
                backoff_until: None,
            })
            .collect();
        if sources.is_empty() {
            let _ = tx.send(Err("No URLs configured".to_string()));
            return;
        }
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

        let sleep = if refresh.is_zero() {
            Duration::from_millis(200)
        } else {
            refresh
        };

        let mut current = 0usize;
        loop {
            let now = Instant::now();

            // Find next source that is not in backoff.
            let mut checked = 0usize;
            while checked < sources.len()
                && sources
                    .get(current)
                    .and_then(|s| s.backoff_until)
                    .is_some_and(|until| until > now)
            {
                current = (current + 1) % sources.len();
                checked += 1;
            }

            let src = &mut sources[current];
            let url = src.url.clone();
            let outcome = fetch_once(&client, &url, api_key.as_deref(), api_key_header.as_deref());

            match outcome {
                FetchResult::Ok(data) => {
                    src.attempts = 0;
                    src.backoff_until = None;
                    if tx.send(Ok(data)).is_err() {
                        debug!("receiver dropped, exiting fetcher");
                        break;
                    }
                }
                FetchResult::Err {
                    message,
                    retry_after,
                } => {
                    src.attempts = src.attempts.saturating_add(1);
                    let backoff = retry_after.unwrap_or_else(|| backoff_duration(src.attempts));
                    src.backoff_until = Some(now + backoff);
                    if tx.send(Err(message)).is_err() {
                        debug!("receiver dropped, exiting fetcher");
                        break;
                    }
                    if sources.len() > 1 {
                        current = (current + 1) % sources.len();
                        debug!(
                            "switching to source {} (backoff {:?})",
                            sources[current].url, backoff
                        );
                    }
                }
            }

            thread::sleep(sleep);
        }
    });
}

fn fetch_once(
    client: &reqwest::blocking::Client,
    url: &str,
    api_key: Option<&str>,
    api_key_header: Option<&str>,
) -> FetchResult {
    let mut req = client.get(url);
    if let (Some(key), Some(header)) = (api_key, api_key_header) {
        if !key.trim().is_empty() && !header.trim().is_empty() {
            req = req.header(header, key);
        }
    }
    let resp = match req.send() {
        Ok(resp) => resp,
        Err(err) => {
            return FetchResult::Err {
                message: err.to_string(),
                retry_after: None,
            }
        }
    };

    let status = resp.status();
    if !status.is_success() {
        let retry_after =
            retry_after_header(resp.headers()).or_else(|| parse_retry_after_msg(resp));
        let message = format!("HTTP {}", status);
        return FetchResult::Err {
            message,
            retry_after,
        };
    }

    match resp.json::<ApiResponse>() {
        Ok(data) => FetchResult::Ok(data),
        Err(err) => FetchResult::Err {
            message: err.to_string(),
            retry_after: None,
        },
    }
}

#[derive(Debug)]
enum FetchResult {
    Ok(ApiResponse),
    Err {
        message: String,
        retry_after: Option<Duration>,
    },
}

fn retry_after_header(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_retry_after_value)
}

fn parse_retry_after_value(value: &str) -> Option<Duration> {
    value.trim().parse::<u64>().ok().map(Duration::from_secs)
}

fn parse_retry_after_msg(resp: reqwest::blocking::Response) -> Option<Duration> {
    // If Retry-After header is absent, try to parse from an error string if available.
    if let Ok(text) = resp.text() {
        if let Some(idx) = text.to_ascii_lowercase().find("retry-after=") {
            let tail = &text[idx + "retry-after=".len()..];
            if let Some(end) = tail.find(|c: char| [' ', ';', '\n'].contains(&c)) {
                if let Ok(secs) = tail[..end].trim_end_matches('s').parse::<u64>() {
                    return Some(Duration::from_secs(secs));
                }
            } else if let Ok(secs) = tail.trim_end_matches('s').parse::<u64>() {
                return Some(Duration::from_secs(secs));
            }
        }
    }
    None
}

fn backoff_duration(attempts: u32) -> Duration {
    if attempts == 0 {
        return Duration::from_secs(0);
    }
    let shift = attempts.saturating_sub(1).min(6);
    let base_secs = 1u64 << shift; // 1,2,4,8,16,32,64
                                   // Simple deterministic jitter without extra deps.
    let jitter_ms = (attempts as u64 * 173) % 1000;
    Duration::from_secs(base_secs.min(60)).saturating_add(Duration::from_millis(jitter_ms))
}

#[cfg(all(test, feature = "net-tests"))]
mod tests {
    use super::{fetch_once, FetchResult};
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
            let result = fetch_once(&client, &url, None, None);
            match result {
                FetchResult::Ok(data) => {
                    assert_eq!(data.now, Some(1));
                    assert_eq!(data.messages, Some(2));
                    assert!(data.aircraft.is_empty());
                }
                _ => panic!("expected ok result"),
            }
        } else {
            let url = "http://127.0.0.1:1";
            let result = fetch_once(&client, url, None, None);
            match result {
                FetchResult::Err { .. } => {}
                _ => panic!("expected error"),
            }
        }
    }
}
