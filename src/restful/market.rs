use super::models::{HistoryCandles, MarketBooks, MarketTicker, MarketTickers, RestApi};
use crate::client::OkxClient;
use anyhow::Result;
use std::collections::BTreeMap;

impl OkxClient {
    // 获取所有产品行情信息
    // GET /api/v5/market/tickers
    pub async fn market_tickers<T>(
        &self,
        inst_type: T,
        uly: Option<T>,
        inst_family: Option<T>,
    ) -> Result<RestApi<MarketTickers>>
    where
        T: Into<String>,
    {
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        if let Some(uly) = uly {
            params.insert("uly".into(), uly.into());
        }
        if let Some(inst_family) = inst_family {
            params.insert("instFamily".into(), inst_family.into());
        }
        params.insert("instType".into(), inst_type.into());
        Ok(self
            .get::<RestApi<MarketTickers>>("/api/v5/market/tickers", &params)
            .await?)
    }

    // 获取单个产品行情信息
    // 获取产品行情信息
    // GET /api/v5/market/ticker
    pub async fn market_ticker<T>(&self, inst_id: T) -> Result<RestApi<MarketTicker>>
    where
        T: Into<String>,
    {
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("instId".into(), inst_id.into());
        Ok(self
            .get::<RestApi<MarketTicker>>("/api/v5/market/ticker", &params)
            .await?)
    }

    // 获取深度
    // api/v5/market/books
    pub async fn market_books<T>(&self, inst_id: T, sz: Option<T>) -> Result<RestApi<MarketBooks>>
    where
        T: Into<String>,
    {
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("instId".into(), inst_id.into());
        if let Some(sz) = sz {
            params.insert("sz".into(), sz.into());
        }
        Ok(self
            .get::<RestApi<MarketBooks>>("/api/v5/market/books", &params)
            .await?)
    }

    // 获取历史行情数据
    // GET api/v5/market/history-candles
    pub async fn market_history_candles<T>(
        &self,
        inst_id: T,
        after: Option<String>,
        before: Option<String>,
        bar: Option<String>,
        limit: Option<T>,
    ) -> Result<RestApi<HistoryCandles>>
    where
        T: Into<String>,
    {
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("instId".into(), inst_id.into());
        // params.insert("after".into(), after.to_string());
        // params.insert("before".into(), before.to_string());
        if let Some(bar) = bar {
            params.insert("bar".into(), bar.into());
        }
        if let Some(limit) = limit {
            params.insert("limit".into(), limit.into());
        }
        Ok(self
            .get::<RestApi<HistoryCandles>>("/api/v5/market/history-candles", &params)
            .await?)
    }

    // 获取产品历史行情
    pub async fn market_candles<T>(
        &self,
        inst_id: T,
        bar: Option<String>,
    ) -> Result<RestApi<HistoryCandles>>
    where
        T: Into<String>,
    {
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("instId".into(), inst_id.into());
        if let Some(bar) = bar {
            params.insert("bar".into(), bar.into());
        }
        Ok(self
            .get::<RestApi<HistoryCandles>>("/api/v5/market/candles", &params)
            .await?)
    }
}
