//! Feed messages received from server.
//!
//! Currently MaiCoin MAX supports the following feeds:
//!
//! - Public orderbooks ([`PubOrderBookFeed`])
//! - Public trades ([`PubTradeFeed`])
//! - Public tickers ([`PubTickerFeed`])
//! - Private orderbooks ([`PrivOrderBookFeed`])
//! - Private trades ([`PrivTradeFeed`])
//! - Private balance changes ([`PrivBalanceFeed`])
//!
//! Each feeds implement [`Feed`] trait, which makes it easy to be dispatched by [`crate::v2::ws::ServerPushEvent`].

use std::result::Result as StdResult;

use chrono::serde as chrono_serde;
use rust_decimal::Decimal;
use serde::{de, de::DeserializeOwned, Deserialize};
use serde_json::Value as JsonValue;

use crate::common::*;
use crate::error::*;

// ========================
// Interfaces and Utilities
// ========================

/// Common interface for feed events pushed by server.
pub trait Feed
where
    Self: Sized + DeserializeOwned,
{
    /// Feed content data.
    type Records;

    /// Returns whether current feed event is a snapshot, or an update.
    fn is_snapshot(&self) -> bool;

    /// Transform the feed into the records it contains.
    fn into_record(self) -> Self::Records;

    /// Deserialize a serde_json::Value into a feed event. You are unlikely to need to work with this directly except via
    /// [`crate::v2::ws::ServerPushEvent`].
    fn from_json_value(value: JsonValue) -> Result<Self> {
        serde_json::from_value::<Self>(value).map_err(Error::WsApiParse)
    }
}

fn parse_pub_feed_type<'de, D>(deserializer: D) -> StdResult<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let val: String = Deserialize::deserialize(deserializer)?;
    match val.to_lowercase().as_str() {
        "snapshot" => Ok(true),
        "update" => Ok(false),
        _ => Err(de::Error::invalid_value(
            de::Unexpected::Str(val.as_str()),
            &"snapshot/update",
        )),
    }
}

fn parse_priv_feed_type<'de, D>(deserializer: D) -> StdResult<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let val: String = Deserialize::deserialize(deserializer)?;
    match val.to_lowercase().as_str() {
        s if s.ends_with("_snapshot") => Ok(true),
        s if s.ends_with("_update") => Ok(false),
        _ => Err(de::Error::invalid_value(
            de::Unexpected::Str(val.as_str()),
            &"*_snapshot/*_update",
        )),
    }
}

// ==================================
// Orderbook feed from public channel
// ==================================

/// Orderbook feed from public channel.
///
/// [Official document](https://maicoin.github.io/max-websocket-docs/#/public_orderbook?id=orderbook-subscription)
#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PubOrderBookFeed {
    /// `true` if this feed is a snapshot.
    #[serde(rename = "e", deserialize_with = "parse_pub_feed_type")]
    pub is_snapshot: bool,
    /// Market name.
    #[serde(rename = "M")]
    pub market: Symbol,
    /// List of ask orders.
    #[serde(rename = "a")]
    pub ask: Vec<PubOrderBookRec>,
    /// List of bid orders.
    #[serde(rename = "b")]
    pub bid: Vec<PubOrderBookRec>,
    /// Timestamp.
    #[serde(rename = "T", with = "chrono_serde::ts_milliseconds")]
    pub time: DateTime,
}

impl Feed for PubOrderBookFeed {
    type Records = (Vec<PubOrderBookRec>, Vec<PubOrderBookRec>);

    fn is_snapshot(&self) -> bool {
        self.is_snapshot
    }

    fn into_record(self) -> Self::Records {
        (self.ask, self.bid)
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PubOrderBookRec {
    pub price: Decimal,
    pub volume: Decimal,
}

// ==============================
// Trade feed from public channel
// ==============================

/// Trade feed from public channel.
///
/// [Official document](https://maicoin.github.io/max-websocket-docs/#/public_trade?id=trade-subscription)
#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PubTradeFeed {
    /// `true` if this feed is a snapshot.
    #[serde(rename = "e", deserialize_with = "parse_pub_feed_type")]
    pub is_snapshot: bool,
    /// Market name.
    #[serde(rename = "M")]
    pub market: Symbol,
    /// List of filled trades.
    #[serde(rename = "t")]
    pub trades: Vec<PubTradeRec>,
    /// Timestamp.
    #[serde(rename = "T", with = "chrono_serde::ts_milliseconds")]
    pub time: DateTime,
}

impl Feed for PubTradeFeed {
    type Records = Vec<PubTradeRec>;

    fn is_snapshot(&self) -> bool {
        self.is_snapshot
    }

    fn into_record(self) -> Self::Records {
        self.trades
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PubTradeRec {
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "v")]
    pub volume: Decimal,
    #[serde(rename = "T", with = "chrono_serde::ts_milliseconds")]
    pub create_time: DateTime,
    #[serde(rename = "tr")]
    pub trend: String,
}

// ===============================
// Ticker feed from public channel
// ===============================

/// Ticker feed from public channel.
///
/// [Official document](https://maicoin.github.io/max-websocket-docs/#/public_ticker?id=ticker-subscription)
#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PubTickerFeed {
    /// `true` if this feed is a snapshot.
    #[serde(rename = "e", deserialize_with = "parse_pub_feed_type")]
    pub is_snapshot: bool,
    /// Market name.
    #[serde(rename = "M")]
    pub market: Symbol,
    /// Ticker (OHLC).
    #[serde(rename = "tk")]
    pub tick: TickerRec,
    /// Timestamp
    #[serde(rename = "T", with = "chrono_serde::ts_milliseconds")]
    pub time: DateTime,
}

impl Feed for PubTickerFeed {
    type Records = TickerRec;

    fn is_snapshot(&self) -> bool {
        self.is_snapshot
    }

    fn into_record(self) -> Self::Records {
        self.tick
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct TickerRec {
    #[serde(rename = "O")]
    pub open: Decimal,
    #[serde(rename = "H")]
    pub close: Decimal,
    #[serde(rename = "L")]
    pub high: Decimal,
    #[serde(rename = "C")]
    pub low: Decimal,
    #[serde(rename = "v")]
    pub volume: Decimal,
}

// ===============================
// Market status feed from public channel
// ===============================

/// Market status feed from public channel.
///
/// [Official document](https://maicoin.github.io/max-websocket-docs/#/public_market_status)
#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PubMarketStatueFeed {
    /// `true` if this feed is a snapshot.
    #[serde(rename = "c")]
    pub channel: String,
    #[serde(rename = "e", deserialize_with = "parse_pub_feed_type")]
    pub is_snapshot: bool,
    /// Market name.
    #[serde(rename = "ms")]
    pub markets: Vec<MarketStatusInfo>,
}

impl Feed for PubMarketStatueFeed {
    type Records = Vec<MarketStatusInfo>;

    fn is_snapshot(&self) -> bool {
        self.is_snapshot
    }

    fn into_record(self) -> Self::Records {
        self.markets
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct MarketStatusInfo {
    #[serde(rename = "M")]
    pub market: String,
    #[serde(rename = "st")]
    pub status: String,
    #[serde(rename = "bu")]
    pub base_unit: String,
    #[serde(rename = "bup")]
    pub base_unit_precision: i8,
    #[serde(rename = "mba")]
    pub min_base_amount: Decimal,
    #[serde(rename = "qu")]
    pub quote_unit: String,
    #[serde(rename = "qup")]
    pub quote_unit_precision: i8,
    #[serde(rename = "mqa")]
    pub min_quote_amount: Decimal,
    #[serde(rename = "mws")]
    pub m_wallet_supported: bool,
}

// ===================================================
// Orderbook feed from private (authenticated) channel
// ===================================================

/// Orderbook feed from private (authenticated) channel.
///
/// [Official document](https://maicoin.github.io/max-websocket-docs/#/private_channels?id=order-response)
#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PrivOrderBookFeed {
    /// `true` if this feed is a snapshot.
    #[serde(rename = "e", deserialize_with = "parse_priv_feed_type")]
    pub is_snapshot: bool,
    /// List of submitted orders.
    #[serde(rename = "o")]
    pub orders: Vec<PrivOrderBookRec>,
    /// Timestamp.
    #[serde(rename = "T", with = "chrono_serde::ts_milliseconds")]
    pub time: DateTime,
}

impl Feed for PrivOrderBookFeed {
    type Records = Vec<PrivOrderBookRec>;

    fn is_snapshot(&self) -> bool {
        self.is_snapshot
    }

    fn into_record(self) -> Self::Records {
        self.orders
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PrivOrderBookRec {
    /// Order ID.
    #[serde(rename = "i")]
    pub oid: u64,
    /// Order side.
    #[serde(rename = "sd")]
    pub side: String,
    /// Order type.
    #[serde(rename = "ot")]
    pub ord_type: String,
    /// Order price.
    #[serde(rename = "p")]
    pub price: Option<Decimal>,
    /// Stop price.
    #[serde(rename = "sp")]
    pub stop_price: Option<Decimal>,
    /// Average price.
    #[serde(rename = "ap")]
    pub avg_price: Option<Decimal>,
    /// Order state.
    #[serde(rename = "S")]
    pub state: String,
    /// Market name.
    #[serde(rename = "M")]
    pub market: Symbol,
    /// Order create time.
    #[serde(rename = "T", with = "chrono_serde::ts_milliseconds")]
    pub create_time: DateTime,
    /// Volume.
    #[serde(rename = "v")]
    pub volume: Decimal,
    /// Remaining volume.
    #[serde(rename = "rv")]
    pub remaining_volume: Option<Decimal>,
    /// Executed volume.
    #[serde(rename = "ev")]
    pub executed_volume: Option<Decimal>,
    /// Trade count.
    #[serde(rename = "tc")]
    pub trade_count: Option<u64>,
    /// Client order ID.
    #[serde(rename = "ci")]
    pub client_oid: Option<String>,
    /// Group ID.
    #[serde(rename = "gi")]
    pub group_id: Option<u64>,
}

// ===============================================
// Trade feed from private (authenticated) channel
// ===============================================

/// Trade feed from private (authenticated) channel.
///
/// [Official document](https://maicoin.github.io/max-websocket-docs/#/private_channels?id=trade-response)
#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PrivTradeFeed {
    /// `true` if this feed is a snapshot.
    #[serde(rename = "e", deserialize_with = "parse_priv_feed_type")]
    pub is_snapshot: bool,
    /// List of filled trades.
    #[serde(rename = "t")]
    pub trades: Vec<PrivTradeRec>,
    /// Timestamp.
    #[serde(rename = "T", with = "chrono_serde::ts_milliseconds")]
    pub time: DateTime,
}

impl Feed for PrivTradeFeed {
    type Records = Vec<PrivTradeRec>;

    fn is_snapshot(&self) -> bool {
        self.is_snapshot
    }

    fn into_record(self) -> Self::Records {
        self.trades
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PrivTradeRec {
    /// Trade ID.
    #[serde(rename = "i")]
    pub tid: u64,
    /// Trade side.
    #[serde(rename = "sd")]
    pub side: String,
    /// Trade price.
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Trade volume.
    #[serde(rename = "v")]
    pub volume: Decimal,
    /// Market name.
    #[serde(rename = "M")]
    pub market: Symbol,
    /// Create time.
    #[serde(rename = "T", with = "chrono_serde::ts_milliseconds")]
    pub create_time: DateTime,
    /// Trade fee.
    #[serde(rename = "f")]
    pub fee: Decimal,
    /// Trade fee currency.
    #[serde(rename = "fc")]
    pub fee_currency: String,
    /// Is trade maker or not.
    #[serde(rename = "m")]
    pub is_maker: bool,
}

// =============================================================
// Balance information feed from private (authenticated) channel
// =============================================================

/// Balance information feed from private (authenticated) channel.
///
/// [Official document](https://maicoin.github.io/max-websocket-docs/#/private_channels?id=account-response)
#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PrivBalanceFeed {
    /// `true` if this feed is a snapshot.
    #[serde(rename = "e", deserialize_with = "parse_priv_feed_type")]
    pub is_snapshot: bool,
    /// Balance for each wallets.
    #[serde(rename = "B")]
    pub balance: Vec<PrivBalanceItem>,
    /// Timestamp.
    #[serde(rename = "T", with = "chrono_serde::ts_milliseconds")]
    pub time: DateTime,
}

impl Feed for PrivBalanceFeed {
    type Records = Vec<PrivBalanceItem>;

    fn is_snapshot(&self) -> bool {
        self.is_snapshot
    }

    fn into_record(self) -> Self::Records {
        self.balance
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct PrivBalanceItem {
    /// Currency name.
    #[serde(rename = "cu")]
    pub currency: String,
    /// Available balance.
    #[serde(rename = "av")]
    pub available: Decimal,
    /// Locked amount.
    #[serde(rename = "l")]
    pub locked: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pub_feed_type_parse() {
        fn parse(input: &str) -> StdResult<bool, serde_json::Error> {
            let mut deserializer = serde_json::Deserializer::from_str(input);
            parse_pub_feed_type(&mut deserializer)
        }

        assert!(parse(r#""snapshot""#).expect("invalid test case"));
        assert!(!parse(r#""update""#).expect("invalid test case"));

        const ERROR_MSG: &str = "must not allow value other than snapshot and update";
        parse(r#""?""#).expect_err(ERROR_MSG);
        parse(r#""_snapshot""#).expect_err(ERROR_MSG);
        parse(r#"" update""#).expect_err(ERROR_MSG);
        parse(r#""""#).expect_err(ERROR_MSG);
        parse(r#""updatesnapshot""#).expect_err(ERROR_MSG);
    }

    #[test]
    fn test_priv_feed_type_parse() {
        fn parse(input: &str) -> StdResult<bool, serde_json::Error> {
            let mut deserializer = serde_json::Deserializer::from_str(input);
            parse_priv_feed_type(&mut deserializer)
        }

        assert!(parse(r#""order_snapshot""#).expect("invalid test case"));
        assert!(!parse(r#""order_update""#).expect("invalid test case"));
        assert!(parse(r#""trade_snapshot""#).expect("invalid test case"));
        assert!(!parse(r#""trade_update""#).expect("invalid test case"));
        assert!(parse(r#""account_snapshot""#).expect("invalid test case"));
        assert!(!parse(r#""account_update""#).expect("invalid test case"));
        assert!(parse(r#""*_snapshot""#).expect("invalid test case"));
        assert!(!parse(r#""??_update""#).expect("invalid test case"));
        assert!(parse(r#""_snapshot""#).expect("invalid test case"));
        assert!(!parse(r#"" _update""#).expect("invalid test case"));

        const ERROR_MSG: &str = "must not allow value other than snapshot and update";
        parse(r#""?""#).expect_err(ERROR_MSG);
        parse(r#""order_snapshot_""#).expect_err(ERROR_MSG);
        parse(r#""order update""#).expect_err(ERROR_MSG);
        parse(r#""order""#).expect_err(ERROR_MSG);
        parse(r#""""#).expect_err(ERROR_MSG);
        parse(r#""updatesnapshot""#).expect_err(ERROR_MSG);
    }
}
