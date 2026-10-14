# SDK Review Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Correct the four API contract defects reproduced against Bybit testnet while preserving type-safe request construction.

**Architecture:** Introduce convert-specific wire types in the asset model, keep open-ended response labels as strings, and correct the two public method signatures. Regression tests exercise real serde behavior and query-pair construction without network mocks.

**Tech Stack:** Rust 2021, serde/serde_json, reqwest query encoding, cargo test/clippy/fmt.

---

### Task 1: Convert-specific request and response types

**Files:**
- Modify: `src/models/asset.rs`
- Modify: `src/api/asset.rs`
- Test: `src/client.rs`

- [x] **Step 1: Write failing convert wire-format tests**

Add tests to `client::response_shape_tests` that require the new public enums,
verify `ConvertAccountType::Uta` serializes as `"eb_convert_uta"`, verify
`ConvertCoinSide` renders as `0`/`1`, and deserialize a convert-history result
containing `"accountType":"funding"`.

```rust
assert_eq!(serde_json::to_value(ConvertAccountType::Uta).unwrap(), "eb_convert_uta");
assert_eq!(ConvertCoinSide::FromCoin.to_string(), "0");
assert_eq!(ConvertCoinSide::ToCoin.to_string(), "1");
let history: ConvertHistoryQueryResponse = serde_json::from_value(serde_json::json!({
    "list": [{"accountType": "funding"}]
})).unwrap();
assert_eq!(history.list.unwrap()[0].account_type.as_deref(), Some("funding"));
```

- [x] **Step 2: Run the focused test and verify RED**

Run: `cargo test convert_types_match_wire_contract`

Expected: compilation fails because `ConvertAccountType` and
`ConvertCoinSide` do not exist and the history field is still `AccountType`.

- [x] **Step 3: Implement the dedicated convert types**

In `src/models/asset.rs`, add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvertAccountType {
    #[serde(rename = "eb_convert_funding")]
    Funding,
    #[serde(rename = "eb_convert_uta")]
    Uta,
    #[serde(rename = "eb_convert_spot")]
    Spot,
    #[serde(rename = "eb_convert_contract")]
    Contract,
    #[serde(rename = "eb_convert_inverse")]
    Inverse,
}

impl std::fmt::Display for ConvertAccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = serde_json::to_value(self).map_err(|_| std::fmt::Error)?;
        f.write_str(value.as_str().ok_or(std::fmt::Error)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertCoinSide { FromCoin, ToCoin }

impl std::fmt::Display for ConvertCoinSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self { Self::FromCoin => "0", Self::ToCoin => "1" })
    }
}
```

Replace convert request uses of `AccountType` with `ConvertAccountType`, replace
`coin_list_query`'s `Side` with `ConvertCoinSide`, and change convert-history
response account fields to `Option<String>`.

- [x] **Step 4: Run focused and full tests and verify GREEN**

Run: `cargo test convert_types_match_wire_contract && cargo test --all-targets`

Expected: the focused test and existing suite pass.

- [x] **Step 5: Commit the convert fix**

```bash
git add src/models/asset.rs src/api/asset.rs src/client.rs
git commit -m "fix: use convert-specific API types"
```

### Task 2: Asset overview open-ended labels

**Files:**
- Modify: `src/models/asset.rs`
- Modify: `src/api/asset.rs`
- Test: `src/client.rs`

- [x] **Step 1: Write a failing live-shape deserialization test**

```rust
let response: ApiResponse<GetAssetOverviewResponse> = serde_json::from_value(
    serde_json::json!({
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
    })
).unwrap();
let item = &response.result.list.unwrap()[0];
assert_eq!(item.account_type.as_deref(), Some("UnifiedTradingAccount"));
```

- [x] **Step 2: Run the focused test and verify RED**

Run: `cargo test asset_overview_accepts_open_ended_labels`

Expected: deserialization fails on the unknown `AccountType` variant.

- [x] **Step 3: Replace unrelated enums with strings**

Change `AssetOverviewAccountItem.account_type` and
`AssetOverviewCategory.category` to `Option<String>`. Change
`get_asset_overview`'s request filter from `Option<AccountType>` to
`Option<&str>` and push the provided string directly.

- [x] **Step 4: Run focused and full tests and verify GREEN**

Run: `cargo test asset_overview_accepts_open_ended_labels && cargo test --all-targets`

Expected: both commands pass.

- [x] **Step 5: Commit the asset overview fix**

```bash
git add src/models/asset.rs src/api/asset.rs src/client.rs
git commit -m "fix: accept asset overview account labels"
```

### Task 3: Required trade-analysis symbol

**Files:**
- Modify: `src/api/account.rs`
- Test: `src/client.rs`

- [x] **Step 1: Write a compile-level signature regression test**

Create a client and call the method with a plain symbol reference:

```rust
let client = BybitClient::testnet("", "").unwrap();
let _request = client.get_trade_info_for_analysis("BTCUSDT", None, None);
```

- [x] **Step 2: Run the focused test and verify RED**

Run: `cargo test trade_analysis_requires_symbol`

Expected: compilation fails because the current method expects
`Option<&str>`.

- [x] **Step 3: Make symbol required and unconditional**

Change the signature to `symbol: &str`, initialize parameters with
`vec![("symbol", symbol)]`, and remove the optional branch.

- [x] **Step 4: Run focused and full tests and verify GREEN**

Run: `cargo test trade_analysis_requires_symbol && cargo test --all-targets`

Expected: both commands pass.

- [x] **Step 5: Commit the symbol fix**

```bash
git add src/api/account.rs src/client.rs
git commit -m "fix: require trade analysis symbol"
```

### Task 4: Repeated Earn auto-savings coins

**Files:**
- Modify: `src/api/earn.rs`
- Test: `src/api/earn.rs`

- [x] **Step 1: Write a failing query-pair test**

Add a module test that calls a not-yet-existing helper:

```rust
#[test]
fn auto_savings_uses_repeated_coin_parameters() {
    assert_eq!(
        auto_savings_params(Some(&["BTC", "ETH"])),
        vec![("coins", "BTC"), ("coins", "ETH")]
    );
    assert!(auto_savings_params(None).is_empty());
}
```

- [x] **Step 2: Run the focused test and verify RED**

Run: `cargo test auto_savings_uses_repeated_coin_parameters`

Expected: compilation fails because `auto_savings_params` does not exist.

- [x] **Step 3: Add the helper and update the public signature**

```rust
fn auto_savings_params<'a>(coins: Option<&'a [&'a str]>) -> Vec<(&'static str, &'a str)> {
    coins
        .unwrap_or_default()
        .iter()
        .map(|coin| ("coins", *coin))
        .collect()
}
```

Change `list_earn_auto_savings` to accept `Option<&[&str]>`, call the helper,
and pass the resulting repeated pairs to `self.get`.

- [x] **Step 4: Run focused and full tests and verify GREEN**

Run: `cargo test auto_savings_uses_repeated_coin_parameters && cargo test --all-targets`

Expected: both commands pass.

- [x] **Step 5: Commit the Earn fix**

```bash
git add src/api/earn.rs
git commit -m "fix: encode repeated Earn coin filters"
```

### Task 5: Documentation and final verification

**Files:**
- Modify: `RELEASE_NOTES.md`
- Modify: `docs/superpowers/plans/2026-10-14-sdk-review-fixes.md`

- [x] **Step 1: Document corrected signatures**

Add a `Fixes` subsection under `2026-10-12` describing the convert-specific
types, open-ended asset labels, required trade-analysis symbol, and repeated
Earn coin filters.

- [x] **Step 2: Run all local quality gates**

Run:

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets --all-features -- -D warnings
git diff --check
```

Expected: every command exits zero with no warnings or failures.

- [x] **Step 3: Run read-only testnet verification**

Use credentials supplied by the user through non-echoing stdin and call only:

- `get_trade_info_for_analysis("BTCUSDT", None, None)`
- `get_asset_overview(None, None, None)`
- `coin_list_query(ConvertAccountType::Uta, None, None)`
- `convert_history_query(None, Some(1), Some(20))`
- `query_small_asset_list(ConvertAccountType::Uta, None)`
- `list_earn_auto_savings(Some(&["BTC", "ETH"]))`

Expected: all six methods return `Ok`; no POST is sent.

- [x] **Step 4: Commit documentation and plan status**

```bash
git add RELEASE_NOTES.md docs/superpowers/plans/2026-10-14-sdk-review-fixes.md
git commit -m "docs: record SDK review fixes"
```
