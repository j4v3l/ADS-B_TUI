use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use crate::model::ApiResponse;

pub fn spawn_fetcher(
    url: String,
    refresh: Duration,
    insecure: bool,
    tx: Sender<Result<ApiResponse, String>>,
) {
    thread::spawn(move || {
        let client = match reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(insecure)
            .timeout(Duration::from_secs(5))
            .build()
        {
            Ok(client) => client,
            Err(err) => {
                let _ = tx.send(Err(format!("Client error: {err}")));
                return;
            }
        };

        loop {
            let result = fetch_once(&client, &url);
            if tx.send(result).is_err() {
                break;
            }
            if refresh > Duration::from_secs(0) {
                thread::sleep(refresh);
            }
        }
    });
}

fn fetch_once(client: &reqwest::blocking::Client, url: &str) -> Result<ApiResponse, String> {
    let resp = client.get(url).send().map_err(|err| err.to_string())?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }
    resp.json::<ApiResponse>().map_err(|err| err.to_string())
}
