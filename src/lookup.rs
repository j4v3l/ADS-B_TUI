use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::model::ApiResponse;
use reqwest::blocking::Client;
use tracing::{debug, error};

#[derive(Clone, Debug)]
pub enum LookupKind {
    Hex(Vec<String>),
    Callsign(Vec<String>),
    Reg(Vec<String>),
    Type(Vec<String>),
    Squawk(Vec<String>),
    Point { lat: f64, lon: f64, radius: f64 },
    Mil,
    Ladd,
    Pia,
}

#[derive(Clone, Debug)]
pub struct LookupRequest {
    pub kind: LookupKind,
}

#[derive(Debug)]
pub enum LookupMessage {
    Result(ApiResponse),
    Error(String),
}

pub fn spawn_lookup_fetcher(
    base_url: String,
    insecure: bool,
    api_key: Option<String>,
    api_key_header: Option<String>,
    rx: Receiver<LookupRequest>,
    tx: Sender<LookupMessage>,
) {
    thread::spawn(move || {
        let client = match Client::builder()
            .danger_accept_invalid_certs(insecure)
            .timeout(Duration::from_secs(6))
            .build()
        {
            Ok(c) => c,
            Err(err) => {
                error!("lookup client error: {err}");
                let _ = tx.send(LookupMessage::Error(format!("Client error: {err}")));
                return;
            }
        };

        let base = base_url.trim_end_matches('/');
        let base_v2 = format!("{base}/v2");

        while let Ok(req) = rx.recv() {
            let url = build_url(&base_v2, &req.kind);
            let mut call = client.get(&url);
            if let (Some(key), Some(header)) = (api_key.as_deref(), api_key_header.as_deref()) {
                if !key.trim().is_empty() && !header.trim().is_empty() {
                    call = call.header(header, key);
                }
            }

            match call.send() {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        let _ = tx.send(LookupMessage::Error(format!("HTTP {}", status)));
                        continue;
                    }
                    match resp.json::<ApiResponse>() {
                        Ok(data) => {
                            let _ = tx.send(LookupMessage::Result(data));
                        }
                        Err(err) => {
                            let _ = tx.send(LookupMessage::Error(format!("Parse error: {err}")));
                        }
                    }
                }
                Err(err) => {
                    debug!("lookup request error: {err}");
                    let _ = tx.send(LookupMessage::Error(err.to_string()));
                }
            }
        }
    });
}

fn build_url(base_v2: &str, kind: &LookupKind) -> String {
    match kind {
        LookupKind::Hex(values) => format!("{}/hex/{}", base_v2, join(values)),
        LookupKind::Callsign(values) => format!("{}/callsign/{}", base_v2, join(values)),
        LookupKind::Reg(values) => format!("{}/reg/{}", base_v2, join(values)),
        LookupKind::Type(values) => format!("{}/type/{}", base_v2, join(values)),
        LookupKind::Squawk(values) => format!("{}/squawk/{}", base_v2, join(values)),
        LookupKind::Point { lat, lon, radius } => {
            format!("{}/point/{lat}/{lon}/{radius}", base_v2)
        }
        LookupKind::Mil => format!("{}/mil", base_v2),
        LookupKind::Ladd => format!("{}/ladd", base_v2),
        LookupKind::Pia => format!("{}/pia", base_v2),
    }
}

fn join(values: &[String]) -> String {
    values
        .iter()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}
