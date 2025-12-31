use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::ws::model::{Candle, OkxCandleMsg};

pub fn extract_okx_candles(json: &Value) -> Result<OkxCandleMsg> {
    let arg = json.get("arg").ok_or_else(|| anyhow!("no arg"))?;

    let channel = arg
        .get("channel")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("no channel"))?
        .to_string();

    let inst_id = arg
        .get("instId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("no instId"))?
        .to_string();

    let data = json
        .get("data")
        .ok_or_else(|| anyhow!("no data"))?
        .as_array()
        .ok_or_else(|| anyhow!("data not array"))?;

    let mut candles = Vec::with_capacity(data.len());

    for item in data {
        let arr = item.as_array().ok_or_else(|| anyhow!("not array"))?;
        if arr.len() < 9 {
            continue;
        }

        let candle = Candle {
            ts: parse_i64(&arr[0])?,
            open: parse_f64(&arr[1])?,
            high: parse_f64(&arr[2])?,
            low: parse_f64(&arr[3])?,
            close: parse_f64(&arr[4])?,
            volume: parse_f64(&arr[5])?,
            confirm: arr[8].as_str() == Some("1"),
        };
        candles.push(candle);
    }
    Ok(OkxCandleMsg {
        channel,
        inst_id,
        candles,
    })
}

fn parse_i64(v: &Value) -> Result<i64> {
    Ok(v.as_str().ok_or_else(|| anyhow!("str"))?.parse()?)
}

fn parse_f64(v: &Value) -> Result<f64> {
    Ok(v.as_str().ok_or_else(|| anyhow!("str"))?.parse()?)
}
