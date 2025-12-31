#[derive(Debug, Clone)]
pub struct Candle {
    pub ts: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub confirm: bool,
}

#[derive(Debug, Clone)]
pub struct OkxCandleMsg {
    pub channel: String,
    pub inst_id: String,
    pub candles: Vec<Candle>,
}
