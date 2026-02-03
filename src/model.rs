use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ApiResponse {
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub now: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_u64_from_any")]
    pub messages: Option<u64>,
    #[serde(default, alias = "ac")]
    pub aircraft: Vec<Aircraft>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Aircraft {
    #[serde(default)]
    pub hex: Option<String>,
    #[serde(rename = "type")]
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub flight: Option<String>,
    #[serde(default)]
    pub r: Option<String>,
    #[serde(default)]
    pub t: Option<String>,
    #[serde(default)]
    pub desc: Option<String>,
    #[serde(default)]
    #[serde(rename = "ownOp")]
    pub own_op: Option<String>,
    #[serde(default)]
    pub year: Option<String>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub alt_baro: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub alt_geom: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_f64_from_any")]
    pub gs: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_f64_from_any")]
    pub track: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub baro_rate: Option<i64>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default, deserialize_with = "de_opt_f64_from_any")]
    pub nav_qnh: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub nav_altitude_mcp: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_f64_from_any")]
    pub lat: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_f64_from_any")]
    pub lon: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub nic: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub rc: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_f64_from_any")]
    pub seen_pos: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub version: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub nic_baro: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub nac_p: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub nac_v: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub sil: Option<i64>,
    #[serde(default)]
    pub sil_type: Option<String>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub alert: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_i64_from_any")]
    pub spi: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_u64_from_any")]
    pub messages: Option<u64>,
    #[serde(default, deserialize_with = "de_opt_f64_from_any")]
    pub seen: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_f64_from_any")]
    pub rssi: Option<f64>,
}

pub fn seen_seconds(ac: &Aircraft) -> Option<f64> {
    if let Some(seen_pos) = ac.seen_pos {
        Some(seen_pos)
    } else {
        ac.seen
    }
}

fn de_opt_i64_from_any<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                Ok(Some(value))
            } else if let Some(value) = number.as_f64() {
                Ok(Some(value as i64))
            } else {
                Ok(None)
            }
        }
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else if let Ok(value) = trimmed.parse::<i64>() {
                Ok(Some(value))
            } else if let Ok(value) = trimmed.parse::<f64>() {
                Ok(Some(value as i64))
            } else {
                Ok(None)
            }
        }
        Value::Null => Ok(None),
        other => Err(serde::de::Error::custom(format!(
            "expected number or null, got {other}"
        ))),
    }
}

fn de_opt_f64_from_any<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Number(number) => number
            .as_f64()
            .map(Some)
            .ok_or_else(|| serde::de::Error::custom("expected float-compatible number")),
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else if let Ok(value) = trimmed.parse::<f64>() {
                Ok(Some(value))
            } else if let Ok(value) = trimmed.parse::<i64>() {
                Ok(Some(value as f64))
            } else {
                Ok(None)
            }
        }
        Value::Null => Ok(None),
        other => Err(serde::de::Error::custom(format!(
            "expected number or null, got {other}"
        ))),
    }
}

fn de_opt_u64_from_any<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Number(number) => {
            if let Some(value) = number.as_u64() {
                Ok(Some(value))
            } else if let Some(value) = number.as_f64() {
                Ok(Some(value.max(0.0) as u64))
            } else {
                Ok(None)
            }
        }
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else if let Ok(value) = trimmed.parse::<u64>() {
                Ok(Some(value))
            } else if let Ok(value) = trimmed.parse::<f64>() {
                Ok(Some(value.max(0.0) as u64))
            } else {
                Ok(None)
            }
        }
        Value::Null => Ok(None),
        other => Err(serde::de::Error::custom(format!(
            "expected number or null, got {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{seen_seconds, ApiResponse};

    const MOCK: &str = r#"{
        "now": 1769903354,
        "messages": 5262546,
        "aircraft": [
            {
                "hex": "ac6668",
                "type": "adsb_icao",
                "flight": "SWA3576 ",
                "r": "N8987Q",
                "t": "B38M",
                "desc": "BOEING 737 MAX 8",
                "ownOp": "SOUTHWEST AIRLINES CO",
                "alt_baro": 22925,
                "alt_geom": 23200,
                "gs": 347.7,
                "track": 339.64,
                "baro_rate": -1024,
                "category": "A3",
                "nav_qnh": 1013.6,
                "nav_altitude_mcp": 19008,
                "lat": 26.853891,
                "lon": -80.544627,
                "nic": 8,
                "rc": 186,
                "seen_pos": 4.355,
                "version": 2,
                "nic_baro": 1,
                "nac_p": 10,
                "nac_v": 2,
                "sil": 3,
                "sil_type": "perhour",
                "alert": 0,
                "spi": 0,
                "mlat": [],
                "tisb": [],
                "messages": 4669,
                "seen": 4.4,
                "rssi": -3.7
            },
            { "hex": "a716f6" },
            { "hex": "a54118" },
            { "hex": "e80444" },
            { "hex": "ac0048" },
            { "hex": "acf5be" }
        ]
    }"#;

    #[test]
    fn parse_mock_data() {
        let data: ApiResponse = serde_json::from_str(MOCK).unwrap();
        assert_eq!(data.now, Some(1769903354));
        assert_eq!(data.messages, Some(5262546));
        assert_eq!(data.aircraft.len(), 6);

        let first = &data.aircraft[0];
        assert_eq!(first.hex.as_deref(), Some("ac6668"));
        assert_eq!(first.own_op.as_deref(), Some("SOUTHWEST AIRLINES CO"));
        assert_eq!(first.baro_rate, Some(-1024));
        assert_eq!(seen_seconds(first), Some(4.355));
    }

    #[test]
    fn parse_numeric_fallbacks() {
        let data: ApiResponse =
            serde_json::from_str(r#"{"now": 123.9, "messages": "42", "aircraft": []}"#).unwrap();
        assert_eq!(data.now, Some(123));
        assert_eq!(data.messages, Some(42));
    }
}
