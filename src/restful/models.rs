use crate::utils::{de_float_64_from_str, de_float_from_str, de_i64_from_str};
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
pub struct RestApi<T> {
    pub code: String,
    pub msg: String,
    pub data: Vec<T>,
}

// 查看余额
// GET /api/v5/account/balance
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalance {
    pub adj_eq: String,
    pub borrow_froz: String,
    pub details: Vec<AccountBalanceDetail>,
    pub imr: String,
    pub iso_eq: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalanceDetail {
    pub avail_bal: String,
    pub avail_eq: String,
    pub borrow_froz: String,
    pub cash_bal: String,
    pub ccy: String,
    pub acc_avg_px: String,
    pub spot_upl_ratio: String,
    pub spot_bal: String,
    pub open_avg_px: String,
}

// 查看持仓信息
// GET /api/v5/account/positions
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountPositions {
    pub mgn_mode: String, //保证金模式
    pub pos_side: String, //持仓方向
    // 持仓数量，逐仓自主划转模式下，转入保证金后会产生pos为0的仓位
    #[serde(deserialize_with = "de_float_from_str")]
    pub pos: f32,
    // 可平仓数量，适用于 币币杠杆,交割/永续（开平仓模式），期权（交易账户及保证金账户逐仓）。
    #[serde(deserialize_with = "de_float_from_str")]
    pub avail_pos: f32,
}

// 查看历史持仓信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountPositionsHistory {
    pub inst_type: String, //持仓方向
    pub inst_id: String,   //持仓方向
    pub mgn_mode: String,  //持仓方向
    #[serde(rename = "type")]
    pub ptype: String, //持仓方向
    #[serde(deserialize_with = "de_float_from_str")]
    pub pnl: f32, // 平仓收益额
}

// 设置杠杆倍数
// POST /api/v5/account/set-leverage
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSetLeverage {
    pub lever: String,
    pub mgn_mode: String,
    pub inst_id: String,
    pub pos_side: String,
}

// 获取所有产品行情信息
// 获取产品行情信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketTickers {
    pub inst_type: String,
    pub inst_id: String,
    pub last: String,
    #[serde(deserialize_with = "de_float_from_str")]
    pub ask_px: f32,
    pub ask_sz: String,
    #[serde(deserialize_with = "de_float_from_str")]
    pub bid_px: f32,
    pub bid_sz: String,
    pub open24h: String,
    pub high24h: String,
    pub low24h: String,
    pub vol_ccy24h: String,
    pub vol24h: String,
    pub sod_utc0: String,
    pub sod_utc8: String,
    pub ts: String,
}

// 获取单个产品行情信息
// 获取产品行情信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketTicker {
    pub inst_type: String,
    pub inst_id: String,
    pub last: String,
    #[serde(deserialize_with = "de_float_from_str")]
    pub ask_px: f32,
    pub ask_sz: String,
    #[serde(deserialize_with = "de_float_from_str")]
    pub bid_px: f32,
    pub bid_sz: String,
    pub open24h: String,
    pub high24h: String,
    pub low24h: String,
    pub vol_ccy24h: String,
    pub vol24h: String,
    pub sod_utc0: String,
    pub sod_utc8: String,
    pub ts: String,
}

///
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketBooks {
    pub asks: Vec<MarketBooksItemData>,
    pub bids: Vec<MarketBooksItemData>,
    pub ts: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketBooksItemData {
    #[serde(deserialize_with = "de_float_from_str")]
    pub price: f32,
    #[serde(deserialize_with = "de_float_from_str")]
    pub sz: f32,
    #[serde(deserialize_with = "de_float_from_str")]
    pub ignore: f32,
    #[serde(deserialize_with = "de_float_from_str")]
    pub count: f32,
}

// 获取未成交订单列表
// 获取当前账户下所有未成交订单信息
// GET /api/v5/trade/orders-pending
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrdersPending {
    pub inst_type: String,
    pub inst_id: String,
    pub tgt_ccy: String,
    pub ccy: String,
    pub ord_id: String,
    pub cl_ord_id: String,
    pub tag: String,
    pub px: String,
    pub sz: String,
    pub pnl: String,
    pub ord_type: String,
    pub side: String,
    pub pos_side: String,
    pub td_mode: String,
    pub acc_fill_sz: String,

    pub fill_px: String,
    pub trade_id: String,
    pub fill_sz: String,
    pub fill_time: String,
    pub avg_px: String,
    pub state: String,

    pub lever: String,
    pub tp_trigger_px: String,
    pub tp_trigger_px_type: String,
    pub sl_trigger_px: String,
    pub sl_trigger_px_type: String,

    pub sl_ord_px: String,
    pub tp_ord_px: String,
    pub fee_ccy: String,
    pub fee: String,
    pub rebate_ccy: String,
    pub source: String,
    pub rebate: String,
    pub category: String,

    pub reduce_only: String,
    pub quick_mgn_type: String,
    pub u_time: String,
    pub c_time: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrdersHistory {
    pub inst_type: String,
    pub inst_id: String,

    pub ord_type: String,
    //     订单状态
    // canceled：撤单成功
    // filled：完全成交
    pub state: String,
    #[serde(deserialize_with = "de_float_from_str")]
    pub pnl: f32,
}

// 批量撤单
// 撤销未完成的订单，每次最多可以撤销20个订单。请求参数应该按数组格式传递。
// POST /api/v5/trade/cancel-batch-orders
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeCancelBatchOrders {
    pub ord_id: String,    //持仓方向
    pub cl_ord_id: String, //持仓方向
    pub s_code: String,    //持仓方向
    pub s_msg: String,     //持仓方向
}

// 下单
// 只有当您的账户有足够的资金才能下单。
// 该接口支持带单合约的下单，但不支持为带单合约平仓
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrder {
    pub ord_id: String,    //订单ID
    pub cl_ord_id: String, //客户自定义订单ID
    pub s_code: String,    //事件执行结果的code，0代表成功
    pub tag: String,       //订单标签
    pub ts: String,        //系统完成订单请求处理的时间戳，Unix时间戳的毫秒数格式
    pub s_msg: String,     //事件执行失败或成功时的msg
}

// 修改订单
// 修改当前未成交的挂单
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeAmendOrder {
    pub ord_id: String,    //持仓方向
    pub cl_ord_id: String, //持仓方向
    pub req_id: String,    //持仓方向
    pub s_code: String,    //持仓方向
    pub s_msg: String,     //持仓方向
}

// 获取订单信息
// 查订单信息

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrderGet {
    pub inst_type: String,
    pub inst_id: String,
    pub tgt_ccy: String,
    pub ccy: String,
    pub ord_id: String,
    pub cl_ord_id: String,
    pub tag: String,
    pub px: String,
    pub sz: String,
    pub pnl: String,
    pub ord_type: String,
    pub side: String,
    pub pos_side: String,
    pub td_mode: String,
    pub acc_fill_sz: String,
    pub state: String, //订单状态  filled
}

///获取交易产品历史K线数据
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryCandles {
    #[serde(deserialize_with = "de_i64_from_str")]
    pub ts: i64,
    #[serde(deserialize_with = "de_float_64_from_str")]
    pub open: f64,
    #[serde(deserialize_with = "de_float_64_from_str")]
    pub high: f64,
    #[serde(deserialize_with = "de_float_64_from_str")]
    pub low: f64,
    #[serde(deserialize_with = "de_float_64_from_str")]
    pub close: f64,
    #[serde(deserialize_with = "de_float_64_from_str")]
    pub vol: f64,
    #[serde(deserialize_with = "de_float_64_from_str")]
    pub vol_ccy: f64,
    #[serde(deserialize_with = "de_float_64_from_str")]
    pub vol_ccy_quote: f64,
    pub confirm: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosePostion {
    pub inst_id: String,
    pub poss_side: String,
    pub cl_ord_id: String,
    pub tag: String,
}
