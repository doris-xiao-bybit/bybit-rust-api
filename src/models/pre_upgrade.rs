use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPreUpgradeTransactionLogResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPreUpgradeDeliveryRecordResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPreUpgradeSettlementRecordResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPreUpgradeClosedPnlResponse {
    #[serde(default)]
    pub ret_code: Option<i64>,
    #[serde(default)]
    pub ret_msg: Option<String>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPreUpgradeExecutionListResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPreUpgradeOrderHistoryResponse {
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
