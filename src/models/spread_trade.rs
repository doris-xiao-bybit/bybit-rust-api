use serde::{Deserialize, Serialize};
use crate::models::Side;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSpreadInstrumentsInfoResponse {
    #[serde(default)]
    pub ret_code: Option<i64>,
    #[serde(default)]
    pub ret_msg: Option<String>,
    #[serde(default)]
    pub ret_ext_info: Option<serde_json::Value>,
    #[serde(default)]
    pub time: Option<i64>,
    #[serde(default)]
    pub result: Option<SpreadInstrumentsInfoResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpreadInstrumentsInfoResult {
    #[serde(default)]
    pub list: Option<Vec<SpreadInstrument>>,
    #[serde(default)]
    pub next_page_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpreadInstrument {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub contract_type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub base_coin: Option<String>,
    #[serde(default)]
    pub quote_coin: Option<String>,
    #[serde(default)]
    pub settle_coin: Option<String>,
    #[serde(default)]
    pub tick_size: Option<String>,
    #[serde(default)]
    pub min_price: Option<String>,
    #[serde(default)]
    pub max_price: Option<String>,
    #[serde(default)]
    pub lot_size: Option<String>,
    #[serde(default)]
    pub min_size: Option<String>,
    #[serde(default)]
    pub max_size: Option<String>,
    #[serde(default)]
    pub launch_time: Option<String>,
    #[serde(default)]
    pub delivery_time: Option<String>,
    #[serde(default)]
    pub legs: Option<Vec<SpreadLeg>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpreadLeg {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub contract_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSpreadOrderbookResponse {
    #[serde(default)]
    pub ret_code: Option<i64>,
    #[serde(default)]
    pub ret_msg: Option<String>,
    #[serde(default)]
    pub ret_ext_info: Option<serde_json::Value>,
    #[serde(default)]
    pub time: Option<i64>,
    #[serde(default)]
    pub result: Option<SpreadOrderbookResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpreadOrderbookResult {
    #[serde(default)]
    pub s: Option<String>,
    #[serde(default)]
    pub b: Option<Vec<GetSpreadOrderbookOrderbookLevel>>,
    #[serde(default)]
    pub a: Option<Vec<GetSpreadOrderbookOrderbookLevel>>,
    #[serde(default)]
    pub ts: Option<i64>,
    #[serde(default)]
    pub u: Option<i64>,
    #[serde(default)]
    pub seq: Option<i64>,
    #[serde(default)]
    pub cts: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSpreadOrderbookOrderbookLevel {
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSpreadRecentTradesResponse {
    #[serde(default)]
    pub ret_code: Option<i64>,
    #[serde(default)]
    pub ret_msg: Option<String>,
    #[serde(default)]
    pub ret_ext_info: Option<serde_json::Value>,
    #[serde(default)]
    pub time: Option<i64>,
    #[serde(default)]
    pub result: Option<SpreadRecentTradesResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpreadRecentTradesResult {
    #[serde(default)]
    pub list: Option<Vec<SpreadTradeRecord>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpreadTradeRecord {
    #[serde(default)]
    pub exec_id: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub price: Option<String>,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub side: Option<Side>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub seq: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSpreadTickersResponse {
    #[serde(default)]
    pub ret_code: Option<i64>,
    #[serde(default)]
    pub ret_msg: Option<String>,
    #[serde(default)]
    pub ret_ext_info: Option<serde_json::Value>,
    #[serde(default)]
    pub time: Option<i64>,
    #[serde(default)]
    pub result: Option<SpreadTickersResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpreadTickersResult {
    #[serde(default)]
    pub list: Option<Vec<SpreadTicker>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpreadTicker {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub bid_price: Option<String>,
    #[serde(default)]
    pub bid_size: Option<String>,
    #[serde(default)]
    pub ask_price: Option<String>,
    #[serde(default)]
    pub ask_size: Option<String>,
    #[serde(default)]
    pub last_price: Option<String>,
    #[serde(default)]
    pub high_price24h: Option<String>,
    #[serde(default)]
    pub low_price24h: Option<String>,
    #[serde(default)]
    pub prev_price24h: Option<String>,
    #[serde(default)]
    pub volume24h: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSpreadMaxQtyResponse {
    #[serde(default)]
    pub ret_code: Option<i64>,
    #[serde(default)]
    pub ret_msg: Option<String>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub ret_ext_info: Option<serde_json::Value>,
    #[serde(default)]
    pub time: Option<i64>,
}
