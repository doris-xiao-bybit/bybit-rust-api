//! HTTP client for Bybit REST API.

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::{debug, warn};

use crate::auth::{generate_signature, get_timestamp};
use crate::config::ClientConfig;
use crate::constants::*;
use crate::error::{BybitError, Result};

/// JSON keys whose values must be masked before being written to debug logs.
///
/// Match is case-insensitive on the unquoted key text. We deliberately do not
/// try to be exhaustive — anything that looks credential-shaped goes here.
const SENSITIVE_JSON_KEYS: &[&str] = &[
    "secret",
    "password",
    "apiSecret",
    "api_secret",
    "apiKey",
    "api_key",
    "privateKey",
    "private_key",
    "token",
    "sign",
];

/// Mask credential values in a JSON body string before logging.
///
/// This is a defensive last line of defense — typed credential fields should
/// already use [`crate::models::RedactedString`]. The mask is purely textual
/// (substring of a JSON dump), so it works on both serialized request bodies
/// and raw HTTP response text.
fn mask_sensitive(json_body: &str) -> String {
    let mut out = json_body.to_owned();
    for key in SENSITIVE_JSON_KEYS {
        // Match `"<key>":"<anything-not-double-quote>"` (the most common JSON shape).
        // Use a simple linear scan: find `"<key>":"`, then replace up to the next `"`.
        let needle = format!("\"{}\":\"", key);
        let needle_lower = needle.to_lowercase();
        let mut cursor = 0usize;
        loop {
            // Case-insensitive find: scan windows of out from cursor.
            let lower_remaining = out[cursor..].to_lowercase();
            let Some(rel) = lower_remaining.find(&needle_lower) else {
                break;
            };
            let start = cursor + rel + needle.len();
            // Find the closing quote (no escape support — Bybit secrets do not contain `"`).
            let Some(end_rel) = out[start..].find('"') else {
                break;
            };
            let end = start + end_rel;
            out.replace_range(start..end, "***REDACTED***");
            cursor = start + "***REDACTED***".len();
        }
    }
    out
}

/// Build a form-urlencoded query string used both for HMAC signing and the
/// request URL. The signed bytes and the bytes Bybit reconstructs from the
/// wire URL must match exactly, so the same encoder is the only safe source.
fn encode_query(params: &[(&str, &str)]) -> String {
    url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(params.iter().copied())
        .finish()
}

#[cfg(test)]
mod mask_tests {
    use super::mask_sensitive;

    #[test]
    fn masks_secret_field() {
        let input = r#"{"apiKey":"k1","secret":"superSecretValue","other":"plain"}"#;
        let out = mask_sensitive(input);
        assert!(out.contains("***REDACTED***"));
        assert!(!out.contains("superSecretValue"));
        assert!(!out.contains("k1"));
        assert!(out.contains("plain"));
    }

    #[test]
    fn masks_password_case_insensitively() {
        let input = r#"{"Password":"p@ss","apisecret":"x"}"#;
        let out = mask_sensitive(input);
        assert!(!out.contains("p@ss"));
        assert!(!out.contains("\"x\""));
    }

    #[test]
    fn leaves_unrelated_keys_intact() {
        let input = r#"{"category":"linear","symbol":"BTCUSDT"}"#;
        assert_eq!(mask_sensitive(input), input);
    }
}

#[cfg(test)]
mod query_tests {
    use super::encode_query;

    #[test]
    fn empty_params_produce_empty_string() {
        assert_eq!(encode_query(&[]), "");
    }

    #[test]
    fn percent_encodes_base64url_cursor() {
        // Bybit pagination cursors are base64url and routinely contain
        // `+`, `/`, `=`. If the signed string differs from the wire query
        // string by even one byte, Bybit returns `10004 sign error`.
        let qs = encode_query(&[("cursor", "abc+/=def"), ("limit", "50")]);
        assert_eq!(qs, "cursor=abc%2B%2F%3Ddef&limit=50");
    }

    #[test]
    fn preserves_param_order() {
        let qs = encode_query(&[("z", "1"), ("a", "2"), ("m", "3")]);
        assert_eq!(qs, "z=1&a=2&m=3");
    }

    #[test]
    fn encodes_ampersand_in_value() {
        let qs = encode_query(&[("symbol", "BTC&ETH")]);
        assert_eq!(qs, "symbol=BTC%26ETH");
    }
}

#[cfg(test)]
mod response_shape_tests {
    use super::{ApiResponse, BybitClient};
    use crate::models::{
        AmendSpreadOrderParams, CancelAllSpreadOrdersParams, CancelSpreadOrderParams,
        ConvertAccountType, ConvertCoinSide, ConvertHistoryQueryResponse, CreateSpreadOrderParams,
        GetAssetOverviewResponse, GetOptionBaseCoinsResponse, GetPreUpgradeOrderHistoryResponse,
        GetSpreadOrderbookResponse, InsLoanProductInfosResponse, OrderType,
        QueryReferralCodeResponse, Side, TimeInForce,
    };

    #[test]
    fn trade_analysis_requires_symbol() {
        let client = BybitClient::testnet("", "").unwrap();
        let _request = client.get_trade_info_for_analysis("BTCUSDT", None, None);
    }

    #[test]
    fn asset_overview_accepts_open_ended_labels() {
        let response: ApiResponse<GetAssetOverviewResponse> =
            serde_json::from_value(serde_json::json!({
                "retCode": 0,
                "retMsg": "Success",
                "result": {
                    "totalEquity": "1",
                    "list": [{
                        "accountType": "UnifiedTradingAccount",
                        "categories": [{"category": "Easy Earn", "equity": "1"}]
                    }]
                },
                "time": 1
            }))
            .expect("asset overview labels should deserialize");

        let list = response.result.list.unwrap();
        assert_eq!(
            list[0].account_type.as_deref(),
            Some("UnifiedTradingAccount")
        );
        assert_eq!(
            list[0].categories.as_ref().unwrap()[0].category.as_deref(),
            Some("Easy Earn")
        );
    }

    #[test]
    fn convert_types_match_wire_contract() {
        assert_eq!(
            serde_json::to_value(ConvertAccountType::Uta).unwrap(),
            serde_json::json!("eb_convert_uta")
        );
        assert_eq!(ConvertCoinSide::FromCoin.to_string(), "0");
        assert_eq!(ConvertCoinSide::ToCoin.to_string(), "1");

        let history: ConvertHistoryQueryResponse = serde_json::from_value(serde_json::json!({
            "list": [{"accountType": "funding"}]
        }))
        .expect("convert history account labels should deserialize");

        assert_eq!(
            history.list.unwrap()[0].account_type.as_deref(),
            Some("funding")
        );
    }

    #[test]
    fn option_base_coins_deserializes_the_unwrapped_result() {
        let raw = r#"{
            "retCode": 0,
            "retMsg": "success",
            "result": {
                "list": [{
                    "baseCoin": "BTC",
                    "quoteCoin": "USDT",
                    "settleCoin": "USDT",
                    "optionShowName": "BTC-Options",
                    "optionOnlineTime": 1736323200000,
                    "hasSymbol": 1,
                    "underlyingType": 0
                }]
            },
            "retExtInfo": {},
            "time": 1736323200000
        }"#;

        let response: ApiResponse<GetOptionBaseCoinsResponse> =
            serde_json::from_str(raw).expect("response should deserialize");

        let result = serde_json::to_value(response.result).expect("result should serialize");
        assert_eq!(
            result.pointer("/list/0/baseCoin").and_then(|v| v.as_str()),
            Some("BTC")
        );
    }

    #[test]
    fn referral_codes_deserialize_the_unwrapped_result() {
        let raw = r#"{
            "retCode": 0,
            "retMsg": "success",
            "result": {
                "referralCodes": [{
                    "referralCode": "code",
                    "referralLink": "https://example.test/code",
                    "scene": 1
                }]
            },
            "retExtInfo": {},
            "time": 1736323200000
        }"#;

        let response: ApiResponse<QueryReferralCodeResponse> =
            serde_json::from_str(raw).expect("response should deserialize");

        let result = serde_json::to_value(response.result).expect("result should serialize");
        assert_eq!(
            result
                .pointer("/referralCodes/0/referralCode")
                .and_then(|v| v.as_str()),
            Some("code")
        );
    }

    #[test]
    fn generated_service_modules_are_exported_and_spread_levels_deserialize() {
        let _: InsLoanProductInfosResponse = serde_json::json!({"list": []});
        let _: GetPreUpgradeOrderHistoryResponse = serde_json::json!({"list": []});

        let result: GetSpreadOrderbookResponse = serde_json::from_value(serde_json::json!({
            "s": "SOLUSDT_SOL/USDT",
            "b": [["21.0000", "0.1"]],
            "a": [["23.0107", "4.6"]],
            "ts": 1744077242177_i64,
            "u": 46977,
            "seq": 213110,
            "cts": 1744076329043_i64
        }))
        .expect("spread orderbook result should deserialize");

        assert_eq!(result.b.as_ref().map(Vec::len), Some(1));
        assert_eq!(result.b.as_ref().unwrap()[0][0], "21.0000");
    }

    #[test]
    fn spread_order_requests_serialize_required_parameters() {
        let create = CreateSpreadOrderParams {
            symbol: "SOLUSDT_SOL/USDT".into(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            qty: "0.1".into(),
            price: Some("21.0".into()),
            order_link_id: Some("spread-1".into()),
            time_in_force: Some(TimeInForce::GTC),
        };
        assert_eq!(
            serde_json::to_value(create).unwrap(),
            serde_json::json!({
                "symbol": "SOLUSDT_SOL/USDT",
                "side": "Buy",
                "orderType": "Limit",
                "qty": "0.1",
                "price": "21.0",
                "orderLinkId": "spread-1",
                "timeInForce": "GTC"
            })
        );

        let amend = AmendSpreadOrderParams {
            symbol: "SOLUSDT_SOL/USDT".into(),
            order_id: Some("order-1".into()),
            order_link_id: None,
            qty: None,
            price: Some("22.0".into()),
        };
        assert_eq!(
            serde_json::to_value(amend).unwrap(),
            serde_json::json!({
                "symbol": "SOLUSDT_SOL/USDT",
                "orderId": "order-1",
                "price": "22.0"
            })
        );

        let cancel = CancelSpreadOrderParams {
            order_id: Some("order-1".into()),
            order_link_id: None,
        };
        assert_eq!(
            serde_json::to_value(cancel).unwrap(),
            serde_json::json!({"orderId": "order-1"})
        );

        let cancel_all = CancelAllSpreadOrdersParams {
            symbol: None,
            cancel_all: Some(true),
        };
        assert_eq!(
            serde_json::to_value(cancel_all).unwrap(),
            serde_json::json!({"cancelAll": true})
        );
    }
}

/// API response wrapper from Bybit.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    /// Return code (0 = success)
    pub ret_code: i32,
    /// Return message
    pub ret_msg: String,
    /// Response data
    pub result: T,
    /// Extended info
    #[serde(default)]
    #[allow(dead_code)]
    pub ret_ext_info: serde_json::Value,
    /// Server time
    #[allow(dead_code)]
    pub time: u64,
}

/// Bybit HTTP API client.
#[derive(Debug, Clone)]
pub struct BybitClient {
    config: ClientConfig,
    http: reqwest::Client,
}

impl BybitClient {
    /// Create a new client with the given configuration.
    pub fn new(config: ClientConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(BybitError::Http)?;

        Ok(Self { config, http })
    }

    /// Create a new client with API credentials using default settings.
    pub fn with_credentials(
        api_key: impl Into<String>,
        api_secret: impl Into<String>,
    ) -> Result<Self> {
        let config = ClientConfig::builder(api_key, api_secret).build();
        Self::new(config)
    }

    /// Create a new client for testnet.
    pub fn testnet(api_key: impl Into<String>, api_secret: impl Into<String>) -> Result<Self> {
        let config = ClientConfig::builder(api_key, api_secret)
            .base_url(TESTNET)
            .build();
        Self::new(config)
    }

    /// Create a new client for demo environment.
    pub fn demo(api_key: impl Into<String>, api_secret: impl Into<String>) -> Result<Self> {
        let config = ClientConfig::builder(api_key, api_secret)
            .base_url(DEMO)
            .build();
        Self::new(config)
    }

    /// Get the client configuration.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Send a public GET request (no authentication).
    pub async fn get_public<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url, endpoint);

        let response = tokio::time::timeout(
            self.config.timeout,
            self.http.get(&url).query(params).send(),
        )
        .await
        .map_err(|_| BybitError::Timeout)?
        .map_err(BybitError::Http)?;

        self.parse_response(response).await
    }

    /// Send an authenticated GET request.
    pub async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let timestamp = get_timestamp();

        // The HMAC payload MUST be byte-identical to the query string Bybit
        // reconstructs from the wire URL. We pre-encode here and append the
        // result to the URL ourselves (rather than letting reqwest re-encode
        // via `.query()`), so values containing reserved chars — most notably
        // `+`, `/`, `=` in base64url pagination cursors — produce a matching
        // sign instead of a `10004 sign error`.
        let query_string = encode_query(params);

        let signature = generate_signature(
            &self.config.api_secret,
            timestamp,
            &self.config.api_key,
            self.config.recv_window,
            &query_string,
        );

        let url = if query_string.is_empty() {
            format!("{}{}", self.config.base_url, endpoint)
        } else {
            format!("{}{}?{}", self.config.base_url, endpoint, query_string)
        };

        let headers = self.build_auth_headers(timestamp, &signature);

        let response = tokio::time::timeout(
            self.config.timeout,
            self.http.get(&url).headers(headers).send(),
        )
        .await
        .map_err(|_| BybitError::Timeout)?
        .map_err(BybitError::Http)?;

        self.parse_response(response).await
    }

    /// Send an authenticated POST request.
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url, endpoint);
        let timestamp = get_timestamp();

        let body_str = serde_json::to_string(body).map_err(|e| BybitError::Parse(e.to_string()))?;

        let signature = generate_signature(
            &self.config.api_secret,
            timestamp,
            &self.config.api_key,
            self.config.recv_window,
            &body_str,
        );

        let headers = self.build_auth_headers(timestamp, &signature);

        if self.config.debug {
            debug!("POST {} body: {}", url, mask_sensitive(&body_str));
        }

        let response = tokio::time::timeout(
            self.config.timeout,
            self.http
                .post(&url)
                .headers(headers)
                .header(CONTENT_TYPE, "application/json")
                .body(body_str)
                .send(),
        )
        .await
        .map_err(|_| BybitError::Timeout)?
        .map_err(BybitError::Http)?;

        self.parse_response(response).await
    }

    /// Build authentication headers.
    fn build_auth_headers(&self, timestamp: u64, signature: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert(
            HEADER_API_KEY,
            HeaderValue::from_str(&self.config.api_key)
                .unwrap_or_else(|_| HeaderValue::from_static("")),
        );
        headers.insert(
            HEADER_TIMESTAMP,
            HeaderValue::from_str(&timestamp.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("0")),
        );
        headers.insert(
            HEADER_SIGN,
            HeaderValue::from_str(signature).unwrap_or_else(|_| HeaderValue::from_static("")),
        );
        headers.insert(HEADER_SIGN_TYPE, HeaderValue::from_static("2"));
        headers.insert(
            HEADER_RECV_WINDOW,
            HeaderValue::from_str(&self.config.recv_window.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("5000")),
        );

        headers
    }

    /// Parse API response and handle errors.
    async fn parse_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();
        let text = response.text().await.map_err(BybitError::Http)?;

        if self.config.debug {
            debug!(
                "Response status: {}, body: {}",
                status,
                mask_sensitive(&text)
            );
        }

        if !status.is_success() {
            // Try to parse as API error
            if let Ok(api_resp) = serde_json::from_str::<ApiResponse<serde_json::Value>>(&text) {
                return Err(BybitError::Api {
                    code: api_resp.ret_code,
                    msg: api_resp.ret_msg,
                });
            }
            return Err(BybitError::Parse(format!(
                "HTTP {} - {}",
                status.as_u16(),
                text
            )));
        }

        // Parse successful response
        let api_resp: ApiResponse<T> = serde_json::from_str(&text).map_err(|e| {
            warn!("Failed to parse response: {}, body: {}", e, text);
            BybitError::Parse(format!(
                "JSON parse error: {} - body: {}",
                e,
                &text[..text.len().min(200)]
            ))
        })?;

        // Check for API-level errors
        if api_resp.ret_code != 0 {
            return Err(BybitError::Api {
                code: api_resp.ret_code,
                msg: api_resp.ret_msg,
            });
        }

        Ok(api_resp.result)
    }
}
