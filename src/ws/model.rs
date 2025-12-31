use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OkxWsMessage {
    pub arg: Option<OkxArg>,
    pub event: Option<String>,
    pub data: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxArg {
    pub channel: String,
    pub inst_id: String,
}
