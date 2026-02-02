use serde_json::{Map, Value};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info};

#[derive(Clone, Debug)]
pub struct RouteRequest {
    pub callsign: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Clone, Debug)]
pub struct RouteResult {
    pub callsign: String,
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub route: Option<String>,
}

#[derive(Clone, Debug)]
pub enum RouteMessage {
    Results(Vec<RouteResult>),
    Error(String),
}

pub fn spawn_route_fetcher(
    base_url: String,
    route_mode: String,
    route_path: String,
    insecure: bool,
    timeout: Duration,
    tx: Sender<RouteMessage>,
    rx: Receiver<Vec<RouteRequest>>,
) {
    thread::spawn(move || {
        info!("route fetcher started");
        let client = match reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(insecure)
            .timeout(timeout)
            .build()
        {
            Ok(client) => client,
            Err(err) => {
                error!("route client error: {err}");
                let _ = tx.send(RouteMessage::Error(format!("Route client error: {err}")));
                return;
            }
        };

        while let Ok(batch) = rx.recv() {
            let mode = route_mode.to_ascii_lowercase();
            let result = if mode == "tar1090" {
                fetch_tar1090(&client, &base_url, &route_path)
            } else {
                if batch.is_empty() {
                    debug!("route fetch skipped (empty batch)");
                    continue;
                }
                fetch_routeset(&client, &base_url, &batch)
            };

            match result {
                Ok(results) => {
                    debug!("route fetch ok: {} results", results.len());
                    let _ = tx.send(RouteMessage::Results(results));
                }
                Err(err) => {
                    error!("route fetch error: {err}");
                    let _ = tx.send(RouteMessage::Error(err));
                }
            }
        }
    });
}

fn fetch_routeset(
    client: &reqwest::blocking::Client,
    base_url: &str,
    batch: &[RouteRequest],
) -> Result<Vec<RouteResult>, String> {
    let url = format!("{}/api/0/routeset", base_url.trim_end_matches('/'));
    let callsigns: Vec<String> = batch
        .iter()
        .map(|req| req.callsign.trim().to_ascii_uppercase())
        .filter(|cs| !cs.is_empty())
        .collect();
    let mut last_err = None;

    let payloads = vec![
        serde_json::json!({
            "planes": batch.iter().map(|req| {
                serde_json::json!({
                    "callsign": req.callsign.trim().to_ascii_uppercase(),
                    "lat": req.lat,
                    "lng": req.lon
                })
            }).collect::<Vec<_>>()
        }),
        serde_json::json!(callsigns),
        serde_json::json!({ "callsigns": callsigns }),
        serde_json::json!({ "callsign": callsigns }),
    ];

    for payload in payloads {
        match post_payload(client, &url, &payload) {
            Ok(body) => return Ok(parse_routes(body)),
            Err(err) => {
                if is_rate_limited_message(&err) {
                    return Err(err);
                }
                last_err = Some(err)
            }
        }
    }

    Err(last_err.unwrap_or_else(|| "Route request failed".to_string()))
}

fn fetch_tar1090(
    client: &reqwest::blocking::Client,
    base_url: &str,
    route_path: &str,
) -> Result<Vec<RouteResult>, String> {
    let mut url = base_url.trim_end_matches('/').to_string();
    let path = route_path.trim_start_matches('/');
    url.push('/');
    url.push_str(path);

    let resp = client.get(url).send().map_err(|err| err.to_string())?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("Route HTTP {}", status));
    }
    let body: Value = resp.json().map_err(|err| err.to_string())?;
    Ok(parse_routes(body))
}

fn post_payload(
    client: &reqwest::blocking::Client,
    url: &str,
    payload: &Value,
) -> Result<Value, String> {
    let resp = client
        .post(url)
        .json(payload)
        .send()
        .map_err(|err| err.to_string())?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("Route HTTP {}", status));
    }
    let body: Value = resp.json::<Value>().map_err(|err| err.to_string())?;
    Ok(body)
}

fn is_rate_limited_message(message: &str) -> bool {
    let msg = message.to_ascii_lowercase();
    msg.contains(" 429")
        || msg.contains("429 ")
        || msg.contains("too many requests")
        || msg.contains("rate limit")
}

fn parse_routes(body: Value) -> Vec<RouteResult> {
    let mut results = Vec::new();

    if let Some(array) = body.as_array() {
        results.extend(parse_route_array(array));
        return results;
    }

    if let Some(obj) = body.as_object() {
        let keys = ["routes", "route", "data", "planes", "aircraft", "results"];
        for key in keys {
            if let Some(array) = obj.get(key).and_then(|v| v.as_array()) {
                results.extend(parse_route_array(array));
                return results;
            }
        }

        if let Some(routes) = obj.get("routes") {
            if let Some(map) = routes.as_object() {
                let mut map_results = Vec::new();
                for (key, value) in map {
                    if let Some(route) = parse_route_object(value, Some(key)) {
                        map_results.push(route);
                    } else if let Some(text) = value.as_str() {
                        map_results.push(RouteResult {
                            callsign: key.trim().to_string(),
                            origin: None,
                            destination: None,
                            route: Some(text.trim().to_string()),
                        });
                    }
                }
                if !map_results.is_empty() {
                    return map_results;
                }
            }
        }

        let mut map_results = Vec::new();
        let mut has_mapped = false;
        for (key, value) in obj {
            if value.is_object() {
                if let Some(route) = parse_route_object(value, Some(key)) {
                    map_results.push(route);
                    has_mapped = true;
                }
            }
        }
        if has_mapped {
            return map_results;
        }
    }

    results
}

fn parse_route_array(array: &[Value]) -> Vec<RouteResult> {
    array
        .iter()
        .filter_map(|item| parse_route_object(item, None))
        .collect()
}

fn parse_route_object(value: &Value, key_callsign: Option<&String>) -> Option<RouteResult> {
    let obj = value.as_object()?;
    let callsign = extract_string(obj, &["callsign", "call", "flight", "cs"])
        .or_else(|| key_callsign.cloned())
        .map(|v| v.trim().to_string())?;

    let route_text = extract_string(
        obj,
        &[
            "route",
            "flightroute",
            "_airport_codes_iata",
            "airport_codes",
        ],
    );
    let origin = extract_string(obj, &["origin", "orig", "from", "departure", "dep"]);
    let destination = extract_string(obj, &["destination", "dest", "to", "arrival", "arr"]);
    let alt_origin = extract_string(obj, &["airport1", "from_iata", "from_icao"]);
    let alt_dest = extract_string(obj, &["airport2", "to_iata", "to_icao"]);

    let (origin, destination, route_text) =
        match (origin.or(alt_origin), destination.or(alt_dest), route_text) {
            (Some(o), Some(d), r) => (Some(o), Some(d), r),
            (None, None, Some(r)) => {
                if let Some((o, d)) = split_route(&r) {
                    (Some(o), Some(d), Some(r))
                } else {
                    (None, None, Some(r))
                }
            }
            other => other,
        };

    Some(RouteResult {
        callsign: callsign.trim().to_string(),
        origin,
        destination,
        route: route_text,
    })
}

fn extract_string(map: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = map.get(*key) {
            if let Some(text) = value.as_str() {
                let text = text.trim();
                if !text.is_empty() {
                    return Some(text.to_string());
                }
            }
        }
    }
    None
}

fn split_route(route: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = route.split('-').collect();
    if parts.len() == 2 {
        let left = parts[0].trim();
        let right = parts[1].trim();
        if !left.is_empty() && !right.is_empty() {
            return Some((left.to_string(), right.to_string()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{parse_routes, parse_route_object, split_route};
    use serde_json::json;

    #[test]
    fn parse_routes_from_array() {
        let body = json!([
            {
                "callsign": "AAL1",
                "origin": "KJFK",
                "destination": "KMIA",
                "route": "KJFK-KMIA"
            }
        ]);
        let results = parse_routes(body);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].callsign, "AAL1");
        assert_eq!(results[0].origin.as_deref(), Some("KJFK"));
        assert_eq!(results[0].destination.as_deref(), Some("KMIA"));
        assert_eq!(results[0].route.as_deref(), Some("KJFK-KMIA"));
    }

    #[test]
    fn parse_routes_from_map() {
        let body = json!({
            "routes": {
                "DAL2": "KLAX-KATL"
            }
        });
        let results = parse_routes(body);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].callsign, "DAL2");
        assert_eq!(results[0].route.as_deref(), Some("KLAX-KATL"));
    }

    #[test]
    fn parse_route_object_with_alts() {
        let value = json!({
            "call": "TEST3",
            "airport1": "KSFO",
            "airport2": "KSEA"
        });
        let result = parse_route_object(&value, None).unwrap();
        assert_eq!(result.callsign, "TEST3");
        assert_eq!(result.origin.as_deref(), Some("KSFO"));
        assert_eq!(result.destination.as_deref(), Some("KSEA"));
    }

    #[test]
    fn split_route_parsing() {
        assert_eq!(
            split_route("KDEN-KORD"),
            Some(("KDEN".to_string(), "KORD".to_string()))
        );
        assert_eq!(split_route("INVALID"), None);
        assert_eq!(split_route(" - "), None);
    }
}
