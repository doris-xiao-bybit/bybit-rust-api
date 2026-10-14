# SDK Review Fixes Design

## Scope

Fix the four defects reproduced while reviewing `doris-main` against
`upstream/main`:

1. Convert APIs use the unrelated general account and order-side enums.
2. Asset overview uses closed enums for open-ended server labels.
3. Trade analysis exposes the required `symbol` parameter as optional.
4. Earn auto-savings cannot encode repeated `coins` query parameters.

No unrelated endpoint generation, response typing, or account-mutating smoke
tests are included.

## API Design

### Convert types

Add endpoint-specific public enums:

- `ConvertAccountType`, serialized to documented values such as
  `eb_convert_uta`, `eb_convert_funding`, `eb_convert_spot`,
  `eb_convert_contract`, and `eb_convert_inverse`.
- `ConvertCoinSide`, serialized as integer `0` (from-coin list) or `1`
  (to-coin list).

Use `ConvertAccountType` for all new convert and small-balance request fields
that currently use `AccountType`. Convert history responses will keep their
wire value as `String`, because the endpoint also returns values such as
`funding` that are not valid request account types.

The affected APIs are new on this unreleased branch, so their incorrect
signatures will be replaced directly rather than retained as deprecated
overloads.

### Asset overview

Represent `AssetOverviewAccountItem.account_type` and
`AssetOverviewCategory.category` as `Option<String>`. The server returns
product labels such as `UnifiedTradingAccount`, `FundingAccount`, `Earn`,
`CRYPTO`, and `Easy Earn`; these are not the same domains as the general
`AccountType` and trading `Category` enums.

Change the optional request filter to accept `Option<&str>` so callers can
send the endpoint's own account labels without being restricted by an
unrelated enum.

### Required symbol

Change `get_trade_info_for_analysis` from `symbol: Option<&str>` to
`symbol: &str` and always include it in the signed query.

### Repeated Earn coins

Change `list_earn_auto_savings` to accept `Option<&[&str]>`. When present,
append one `("coins", coin)` pair per element, preserving the documented
`coins=BTC&coins=ETH` representation and signature order. `None` continues to
query all coins.

## Error Handling and Compatibility

Existing client signing and percent-encoding remain unchanged. Repeated query
keys are already supported by the parameter slice and form encoder.

These changes intentionally break only the newly introduced, incorrect API
signatures on the current unreleased branch. Existing `upstream/main` APIs are
not changed.

## Testing

Use test-driven development and observe each regression test fail before the
production change:

- Convert request enums serialize to the exact documented wire values.
- Convert history deserializes `accountType: "funding"`.
- Asset overview deserializes the observed testnet labels.
- Trade analysis query construction requires and includes `symbol`.
- Earn auto-savings builds repeated `coins` keys that encode as repeated query
  parameters.

After unit tests pass, run formatting, all targets, doctests, and clippy with
warnings denied. Finally, repeat only the relevant read-only testnet GETs:
trade analysis, asset overview, convert coin list/history, small-asset list,
and multi-coin Earn auto-savings. No POST request will be executed.
