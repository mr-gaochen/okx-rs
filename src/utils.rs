use chrono::format::DelayedFormat;
use chrono::format::StrftimeItems;
use chrono::prelude::*;
use serde::de;
use serde::{Deserialize, Deserializer};

pub async fn get_send_time() -> String {
    let fmt = "%Y-%m-%d %H:%M:%S";
    let now: DateTime<Local> = Local::now();
    let dft: DelayedFormat<StrftimeItems> = now.format(fmt);
    let str_date: String = dft.to_string();
    // 2021-01-04 20:02:09
    str_date
}

pub fn get_unix() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

pub fn get_unix_nano() -> i128 {
    time::OffsetDateTime::now_utc().unix_timestamp_nanos()
}

pub fn default_perent() -> f32 {
    0.0
}

pub fn de_float_from_str<'a, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'a>,
{
    let str_val = String::deserialize(deserializer)?;
    if str_val == "".to_string() {
        return Ok(0.0);
    }
    str_val.parse::<f32>().map_err(de::Error::custom)
}

pub fn de_float_64_from_str<'a, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'a>,
{
    let str_val = String::deserialize(deserializer)?;
    if str_val == "".to_string() {
        return Ok(0.0);
    }
    str_val.parse::<f64>().map_err(de::Error::custom)
}

pub fn de_i64_from_str<'a, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'a>,
{
    let str_val = String::deserialize(deserializer)?;
    if str_val.trim().is_empty() {
        return Ok(0);
    }
    str_val.parse::<i64>().map_err(de::Error::custom)
}
