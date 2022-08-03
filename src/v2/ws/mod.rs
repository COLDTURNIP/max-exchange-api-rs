//! Components to interact with the websocket API: <https://maicoin.github.io/max-websocket-docs>
//!
//! This module provides only the data entries. Users are free to apply these components on any websocket client, and
//! responsible to maintain the socket state, including keeping connection alive.
//!
//! # Example
//!
//! ```no_run
//! # use anyhow::{bail, Result};
//! # use async_std::task;
//! # use async_tungstenite::async_std::connect_async;
//! # use async_tungstenite::tungstenite::Message;
//! # use futures::{sink::SinkExt, stream::StreamExt};
//! use maicoin_max::v2::ws::{ServerPushEvent, SubRequest, BASE_URL};
//!
//! # fn main() -> Result<()> {
//! #     task::block_on(async {
//! let mut stream = connect_async(BASE_URL).await?.0.fuse();
//!
//! // send subscription request
//! let mut sub = SubRequest::new_sub(String::new());
//! sub.subset().insert_ticker("usdttwd".into());
//! let req = serde_json::to_string(&sub)?;
//! stream.send(Message::text(req)).await?;
//!
//! // response of subscription request
//! if let Some(Ok(Message::Text(resp))) = stream.next().await {
//!     match serde_json::from_str::<ServerPushEvent>(&resp)? {
//!         ServerPushEvent::Error(err) => bail!("error while submitting ticker: {:?}", err),
//!         ServerPushEvent::SubResp(_) => {}
//!         event => bail!("unexpected response: {:?}", event),
//!     }
//! }
//!
//! // receiving feed events form server
//! if let Some(Ok(Message::Text(raw))) = stream.next().await {
//!     let event: ServerPushEvent = serde_json::from_str::<ServerPushEvent>(&raw).unwrap();
//!     match event {
//!         ServerPushEvent::Error(err) => println!("error while receiving feed: {:?}", err),
//!         ServerPushEvent::PubTickerFeed(ticker) => println!("{:?}", ticker),
//!         event => bail!("unexpected event: {:?}", event),
//!     }
//! }
//! #         Ok(())
//! #     })
//! # }
//! ```

// Server pushes
pub mod feed;

use std::collections::HashMap;
use std::fmt;
use std::result::Result as StdResult;

use chrono::serde as chrono_serde;
use hmac::{Hmac, Mac, NewMac};
use serde::{
    de,
    de::{SeqAccess, Visitor},
    ser,
    ser::SerializeSeq,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::Sha256;

use crate::common::*;
use crate::error::*;
use crate::Credentials;

// ================
// Public constants
// ================

/// The websocket API base URL.
pub const BASE_URL: &str = "wss://max-stream.maicoin.com/ws";

// ====================
// Client side requests
// ====================

/// Channel subscription/unsubscription requests
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(tag = "action")]
pub enum SubRequest {
    #[serde(rename = "sub")]
    Subscribe {
        subscriptions: SubscribeChannelSet,
        id: String,
    },

    #[serde(rename = "unsub")]
    Unsubscribe {
        subscriptions: SubscribeChannelSet,
        id: String,
    },
}

impl SubRequest {
    pub fn new_sub(id: String) -> Self {
        Self::Subscribe {
            subscriptions: SubscribeChannelSet::new(),
            id,
        }
    }

    pub fn new_unsub(id: String) -> Self {
        Self::Unsubscribe {
            subscriptions: SubscribeChannelSet::new(),
            id,
        }
    }

    pub fn subset(&mut self) -> &mut SubscribeChannelSet {
        match self {
            Self::Subscribe {
                subscriptions: ref mut subset,
                ..
            } => subset,
            Self::Unsubscribe {
                subscriptions: ref mut subset,
                ..
            } => subset,
        }
    }
}

/// Set of channels to subscribe/unsubscribe.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct SubscribeChannelSet(HashMap<(PubChannelType, String), PubChannelDetails>);

/// Subscription types of public channels.
#[derive(Eq, PartialEq, Hash, Debug)]
enum PubChannelType {
    Orderbook, // "orderbook"
    Trade,     // "trade"
    Ticker,    // "ticker"
}

impl ToString for PubChannelType {
    fn to_string(&self) -> String {
        match self {
            Self::Orderbook => "book".into(),
            Self::Trade => "trade".into(),
            Self::Ticker => "ticker".into(),
        }
    }
}

impl std::str::FromStr for PubChannelType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "orderbook" => Ok(Self::Orderbook),
            "book" => Ok(Self::Orderbook),
            "trade" => Ok(Self::Trade),
            "ticker" => Ok(Self::Ticker),
            _ => Err(Error::WsInvalidValue(s.to_owned())),
        }
    }
}

/// Channel subscription details.
#[derive(Serialize, Deserialize, Default, Debug, Eq, PartialEq)]
pub struct PubChannelDetails {
    pub channel: String,
    pub market: Symbol,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
}

impl SubscribeChannelSet {
    pub fn new() -> Self {
        Default::default()
    }

    /// Insert an orderbook subscription.
    pub fn insert_orderbook(&mut self, market: Symbol, depth: Option<u32>) -> bool {
        self.0
            .insert(
                (PubChannelType::Orderbook, market.clone()),
                PubChannelDetails {
                    channel: PubChannelType::Orderbook.to_string(),
                    market,
                    depth,
                },
            )
            .is_none()
    }

    /// Insert a trade subscription.
    pub fn insert_trade(&mut self, market: Symbol) -> bool {
        self.0
            .insert(
                (PubChannelType::Trade, market.clone()),
                PubChannelDetails {
                    channel: PubChannelType::Trade.to_string(),
                    market,
                    ..Default::default()
                },
            )
            .is_none()
    }

    /// Insert a ticker subscription.
    pub fn insert_ticker(&mut self, market: Symbol) -> bool {
        self.0
            .insert(
                (PubChannelType::Ticker, market.clone()),
                PubChannelDetails {
                    channel: PubChannelType::Ticker.to_string(),
                    market,
                    ..Default::default()
                },
            )
            .is_none()
    }

    fn insert_entry(&mut self, entry: PubChannelDetails) -> Result<bool> {
        let mut entry = entry;
        entry.channel = entry.channel.to_lowercase();
        let book_type: PubChannelType = entry.channel.parse()?;
        Ok(self
            .0
            .insert((book_type, entry.market.clone()), entry)
            .is_none())
    }

    /// Insert an orderbook subscription.
    pub fn remove_orderbook(&mut self, market: Symbol) -> bool {
        self.0
            .remove(&(PubChannelType::Orderbook, market))
            .is_some()
    }

    /// Insert a trade subscription.
    pub fn remove_trade(&mut self, market: Symbol) -> bool {
        self.0.remove(&(PubChannelType::Trade, market)).is_some()
    }

    /// Insert a ticker subscription.
    pub fn remove_ticker(&mut self, market: Symbol) -> bool {
        self.0.remove(&(PubChannelType::Ticker, market)).is_some()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = &'_ PubChannelDetails> + '_> {
        Box::new(self.0.iter().map(|(_k, v)| v))
    }
}

impl Serialize for SubscribeChannelSet {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for (_, entry) in self.0.iter() {
            seq.serialize_element(entry)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for SubscribeChannelSet {
    fn deserialize<D>(deserializer: D) -> StdResult<SubscribeChannelSet, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct SubsetSeqVisitor;

        impl<'de> Visitor<'de> for SubsetSeqVisitor {
            type Value = SubscribeChannelSet;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Subset key value sequence.")
            }

            fn visit_seq<A>(self, mut seq: A) -> StdResult<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut new_set = SubscribeChannelSet::new();
                while let Some(entry) = seq.next_element::<PubChannelDetails>()? {
                    new_set
                        .insert_entry(entry)
                        .map_err(|err| de::Error::custom(format!("{:?}", err)))?;
                }

                Ok(new_set)
            }
        }

        let visitor = SubsetSeqVisitor {};
        deserializer.deserialize_seq(visitor)
    }
}

/// Authentication request for private channels.
///
/// [Official document](https://maicoin.github.io/max-websocket-docs/#/authentication)
#[derive(Serialize)]
pub struct AuthRequest {
    action: &'static str,
    #[serde(rename = "apiKey")]
    api_key: String,
    nonce: u64,
    signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<Vec<PrivFeedType>>,
}

/// Types of channels to be subscribe.
///
/// [Official document](https://maicoin.github.io/max-websocket-docs/#/authentication?id=subscription-with-filters)
#[derive(Serialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PrivFeedType {
    Order,
    Trade,
    Account,
    TradeUpdate,
}

impl AuthRequest {
    /// Create authentication request from credentials. Note that the authentication request contains time-based nonce
    /// information. Caller is responsible to send the request out as soon as possible.
    pub fn new(
        credential: &Credentials,
        id: Option<String>,
        filters: Option<Vec<PrivFeedType>>,
    ) -> Self {
        Self::new_with_nonce(
            credential.access_key.as_str(),
            credential.secret_key.as_str(),
            credential.nonce(),
            id,
            filters,
        )
    }

    // Helper constructor for testing.
    fn new_with_nonce(
        key: &str,
        secret: &str,
        nonce: u64,
        id: Option<String>,
        filters: Option<Vec<PrivFeedType>>,
    ) -> Self {
        let mut mac =
            Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("Hmac::new(api_sec)");
        mac.update(nonce.to_string().as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());
        Self {
            action: "auth",
            api_key: key.to_owned(),
            nonce,
            signature,
            id,
            filters,
        }
    }
}

// ===============================
// Event handling from server side
// ===============================

/// Universal server pushed event dispatcher. It wraps the request responses([`SubResponse`], [`AuthResult`]), errors ([`ServerPushError`]), and the feeds defined in [`crate::v2::ws::feed`].
///
/// ```ignore
/// if let Ok(event) = serde_json::from_str::<ServerPushEvent>(received_websocket_packet) {
///     match event {
///         ServerPushEvent::PubOrderbookFeed(feed) => ...(handle order feed)...
///         ServerPushEvent::PubTickerFeed(feed) => ...(handle ticker feed)...
///         unexpected_event => error!("unexpected feed: {:?}", unexpected_event),
///     }
/// } else {
///     error!("failed to parse server event: {}", raw);
/// }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub enum ServerPushEvent {
    /// Errors warned by server
    Error(ServerPushError),
    /// Response of channel subscription
    SubResp(SubResponse),
    /// Response of channel unsubscription
    UnsubResp(SubResponse),
    /// Response socket authentication
    AuthResp(AuthResult),

    /// Server pushed public orderbook feeds
    PubOrderbookFeed(feed::PubOrderBookFeed),
    /// Server pushed public trade feeds
    PubTradeFeed(feed::PubTradeFeed),
    /// Server pushed public ticker feeds
    PubTickerFeed(feed::PubTickerFeed),
    /// Server pushed public market status feeds
    PubMarketStatueFeed(feed::PubMarketStatueFeed),

    /// Server pushed private orderbook feeds
    PrivOrderbookFeed(feed::PrivOrderBookFeed),
    /// Server pushed private trade feeds
    PrivTradeFeed(feed::PrivTradeFeed),
    /// Server pushed private balance changes
    PrivBalanceFeed(feed::PrivBalanceFeed),
}

impl<'de> Deserialize<'de> for ServerPushEvent {
    fn deserialize<D>(deserializer: D) -> StdResult<ServerPushEvent, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let root: JsonValue = Deserialize::deserialize(deserializer)?;
        if root["E"].is_array() {
            serde_json::from_value(root).map(Self::Error)
        } else {
            let event_type = root["e"].as_str().unwrap_or("N/A");
            let channel = root["c"].as_str().unwrap_or("N/A");
            match (event_type, channel) {
                // channel states
                ("subscribed", _) => serde_json::from_value(root).map(Self::SubResp),
                ("unsubscribed", _) => serde_json::from_value(root).map(Self::UnsubResp),
                ("authenticated", _) => serde_json::from_value(root).map(Self::AuthResp),

                // public channels
                (_, "book") => serde_json::from_value(root).map(Self::PubOrderbookFeed),
                (_, "trade") => serde_json::from_value(root).map(Self::PubTradeFeed),
                (_, "ticker") => serde_json::from_value(root).map(Self::PubTickerFeed),
                (_, "market_status") => serde_json::from_value(root).map(Self::PubMarketStatueFeed),

                // private channels
                (et, "user") if et.starts_with("order_") => {
                    serde_json::from_value(root).map(Self::PrivOrderbookFeed)
                }
                (et, "user") if et.starts_with("trade_") => {
                    serde_json::from_value(root).map(Self::PrivTradeFeed)
                }
                (et, "user") if et.starts_with("account_") => {
                    serde_json::from_value(root).map(Self::PrivBalanceFeed)
                }

                _ => {
                    return Err(de::Error::unknown_variant(
                        &format!("{{e: {}, c: {}}}", event_type, channel),
                        &[
                            "(subscribed, N/A)",
                            "(unsubscribed, N/A)",
                            "(authenticated, N/A)",
                            "(snapshot/uppdate, book/trade/ticker)",
                            "(order_*, user)",
                            "(trade_*, user)",
                            "(account_*, user)",
                        ],
                    ))
                }
            }
        }
        .map_err(de::Error::custom)
    }
}

/// Represents error response.
///
/// [Offical document](https://maicoin.github.io/max-websocket-docs/#/?id=error-response)
#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct ServerPushError {
    #[serde(rename = "E")]
    pub msg: Vec<String>,
    #[serde(rename = "i")]
    pub id: String,
    #[serde(rename = "T", with = "chrono_serde::ts_milliseconds")]
    pub time: DateTime,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct SubResponse {
    /// `true` for subscription response, `false` for unsubscription.
    #[serde(
        rename = "e",
        deserialize_with = "SubResponse::parse_sub_resp_sub_unsub"
    )]
    pub is_subscribe: bool,
    /// Channels that subscribed/unsubscribed.
    #[serde(rename = "s")]
    pub subscriptions: SubscribeChannelSet,
    /// Client ID.
    #[serde(rename = "i")]
    pub id: String,
    /// Timestamp.
    #[serde(rename = "T", with = "chrono_serde::ts_milliseconds")]
    pub time: DateTime,
}

impl SubResponse {
    fn parse_sub_resp_sub_unsub<'de, D>(deserializer: D) -> StdResult<bool, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let val: String = Deserialize::deserialize(deserializer)?;
        match val.as_str() {
            "subscribed" => Ok(true),
            "unsubscribed" => Ok(false),
            _ => Err(de::Error::invalid_value(
                de::Unexpected::Str(val.as_str()),
                &"subscribed/unsubscribed",
            )),
        }
    }
}

/// Authenication result.
#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct AuthResult {
    /// Client ID.
    #[serde(rename = "i")]
    pub id: String,
    /// Timestamp.
    #[serde(rename = "T", with = "chrono_serde::ts_milliseconds")]
    pub time: DateTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{SubsecRound, Utc};
    use serde_json::json;

    #[test]
    fn test_reqsub_subscribe_json_serialize_deserialize() {
        let mut orig = SubRequest::new_sub(String::new());
        orig.subset().insert_orderbook("market_A".into(), Some(1));
        orig.subset().insert_orderbook("market_B".into(), None);
        orig.subset().insert_trade("market_C".into());
        orig.subset().insert_ticker("market_D".into());
        let mut result = serde_json::to_value(orig).expect("failed to serialize");
        let result_subset: SubscribeChannelSet =
            serde_json::from_value(result["subscriptions"].take()).expect("failed to deserialize");
        let expect_subset: SubscribeChannelSet = serde_json::from_value(json!([
            {"channel": "book", "market": "market_A", "depth": 1},
            {"channel": "book", "market": "market_B"},
            {"channel": "trade", "market": "market_C"},
            {"channel": "ticker", "market": "market_D"}
        ]))
        .expect("invalid test case");
        assert_eq!(
            result,
            json!({"action": "sub", "id": "", "subscriptions": null})
        );
        assert_eq!(result_subset, expect_subset);
    }

    #[test]
    fn test_reqsub_unsubscribe_json_serialize_deserialize() {
        let mut orig = SubRequest::new_unsub(String::new());
        orig.subset().insert_orderbook("market_A".into(), None);
        orig.subset().insert_orderbook("market_B".into(), Some(100));
        orig.subset().insert_trade("market_C".into());
        orig.subset().insert_ticker(String::new());
        let mut result = serde_json::to_value(orig).expect("failed to serialize");
        let result_subset: SubscribeChannelSet =
            serde_json::from_value(result["subscriptions"].take()).expect("failed to deserialize");
        let expect_subset: SubscribeChannelSet = serde_json::from_value(json!([
            {"channel": "book", "market": "market_A"},
            {"channel": "book", "market": "market_B", "depth": 100},
            {"channel": "trade", "market": "market_C"},
            {"channel": "ticker", "market": ""}
        ]))
        .expect("invalid test case");
        assert_eq!(
            result,
            json!({"action": "unsub", "id": "", "subscriptions": null})
        );
        assert_eq!(result_subset, expect_subset);
    }

    #[test]
    fn test_subchanset_orderbook_add() {
        let mut set = SubscribeChannelSet::new();
        set.insert_orderbook("market_A".into(), Some(3));
        set.insert_orderbook("market_B".into(), Some(0));
        set.insert_orderbook("market_A".into(), None);
        assert_eq!(set.0.len(), 2);
        assert_eq!(
            set.0.get(&(PubChannelType::Orderbook, "market_A".into())),
            Some(&PubChannelDetails {
                channel: "book".into(),
                market: "market_A".into(),
                depth: None,
            })
        );
        assert_eq!(
            set.0.get(&(PubChannelType::Orderbook, "market_B".into())),
            Some(&PubChannelDetails {
                channel: "book".into(),
                market: "market_B".into(),
                depth: Some(0),
            })
        );
    }

    #[test]
    fn test_subchanset_trade_add() {
        let mut set = SubscribeChannelSet::new();
        set.insert_trade("market_A".into());
        set.insert_trade("market_B".into());
        set.insert_trade("market_A".into());
        assert_eq!(set.0.len(), 2);
        assert_eq!(
            set.0.get(&(PubChannelType::Trade, "market_A".into())),
            Some(&PubChannelDetails {
                channel: "trade".into(),
                market: "market_A".into(),
                depth: None,
            })
        );
        assert_eq!(
            set.0.get(&(PubChannelType::Trade, "market_B".into())),
            Some(&PubChannelDetails {
                channel: "trade".into(),
                market: "market_B".into(),
                depth: None,
            })
        );
    }

    #[test]
    fn test_subchanset_ticker_add() {
        let mut set = SubscribeChannelSet::new();
        set.insert_ticker("market_A".into());
        set.insert_ticker("market_B".into());
        set.insert_ticker("market_A".into());
        assert_eq!(set.0.len(), 2);
        assert_eq!(
            set.0.get(&(PubChannelType::Ticker, "market_A".into())),
            Some(&PubChannelDetails {
                channel: "ticker".into(),
                market: "market_A".into(),
                depth: None,
            })
        );
        assert_eq!(
            set.0.get(&(PubChannelType::Ticker, "market_B".into())),
            Some(&PubChannelDetails {
                channel: "ticker".into(),
                market: "market_B".into(),
                depth: None,
            })
        );
    }

    #[test]
    fn test_subchanset_channel_remove() {
        let mut set = SubscribeChannelSet::new();
        set.insert_orderbook("market_A".into(), Some(3));
        set.insert_orderbook("market_B".into(), Some(5));
        set.insert_trade("market_B".into());
        set.insert_ticker("market_A".into());
        set.remove_orderbook("market_A".into());
        set.remove_ticker("market_C".into());
        assert_eq!(set.len(), 3);
        assert_eq!(
            set.0.get(&(PubChannelType::Orderbook, "market_B".into())),
            Some(&PubChannelDetails {
                channel: "book".into(),
                market: "market_B".into(),
                depth: Some(5),
            })
        );
        assert_eq!(
            set.0.get(&(PubChannelType::Trade, "market_B".into())),
            Some(&PubChannelDetails {
                channel: "trade".into(),
                market: "market_B".into(),
                depth: None,
            })
        );
        assert_eq!(
            set.0.get(&(PubChannelType::Ticker, "market_A".into())),
            Some(&PubChannelDetails {
                channel: "ticker".into(),
                market: "market_A".into(),
                depth: None,
            })
        );
    }

    #[test]
    fn test_subchanset_json_serialize_deserialize() {
        let mut orig = SubscribeChannelSet::new();
        orig.insert_orderbook("market_A".into(), Some(3));
        orig.insert_orderbook("market_B".into(), Some(5));
        orig.insert_trade("market_B".into());
        orig.insert_ticker("market_A".into());
        let json_str = serde_json::to_string(&orig).expect("failed to serialize");
        assert!(!json_str.is_empty());
        let result: SubscribeChannelSet =
            serde_json::from_str(&json_str).expect("failed to deserialize");
        assert_eq!(orig, result);
    }

    #[test]
    fn test_auth_request_json_serialize() {
        let orig = AuthRequest::new_with_nonce(
            &"api key",
            &"api secret",
            12345,
            Some("client_id".into()),
            Some(vec![
                PrivFeedType::Trade,
                PrivFeedType::Account,
                PrivFeedType::Order,
                PrivFeedType::TradeUpdate,
            ]),
        );
        let expect = json!({
            "action": "auth",
            "apiKey": "api key",
            "nonce": 12345,
            "signature": "c1a6d487006e3e9d5e0966075e7de7cd5de3681cbcc5946b3876972defc70cb2",
            "id": "client_id",
            "filters": ["trade", "account", "order", "trade_update"]
        });

        let json_str = serde_json::to_string(&orig).expect("failed to serialize");
        assert!(!json_str.is_empty());
        let result = serde_json::from_str::<JsonValue>(&json_str).expect("failed to deserialize");
        assert_eq!(expect, result);
    }

    #[test]
    fn test_error_resp_json_deserialize() {
        let test_time = Utc::now().trunc_subsecs(0);
        let orig = json!({
            "E": ["entry_0", "entry_1", "", "final_entry"],
            "i": "test_client_id",
            "T": test_time.timestamp() * 1000
        });
        let expect = ServerPushError {
            msg: ["entry_0", "entry_1", "", "final_entry"]
                .iter()
                .map(|&s| String::from(s))
                .collect(),
            id: "test_client_id".into(),
            time: test_time,
        };

        let result =
            serde_json::from_value::<ServerPushError>(orig).expect("failed to deserialize");
        assert_eq!(expect, result);
    }

    #[test]
    fn test_sub_resp_json_deserialize() {
        let test_time = Utc::now().trunc_subsecs(0);
        let orig = json!({
            "e": "subscribed",
            "s": [],
            "i": "test_client_id",
            "T": test_time.timestamp() * 1000
        });
        let expect = SubResponse {
            is_subscribe: true,
            subscriptions: SubscribeChannelSet::new(),
            id: "test_client_id".into(),
            time: test_time,
        };

        let result = serde_json::from_value::<SubResponse>(orig).expect("failed to deserialize");
        assert_eq!(expect, result);
    }

    #[test]
    fn test_unsub_resp_json_deserialize() {
        let test_time = Utc::now().trunc_subsecs(0);
        let orig = json!({
            "e": "unsubscribed",
            "s": [],
            "i": "test_client_id",
            "T": test_time.timestamp() * 1000
        });
        let expect = SubResponse {
            is_subscribe: false,
            subscriptions: SubscribeChannelSet::new(),
            id: "test_client_id".into(),
            time: test_time,
        };

        let result = serde_json::from_value::<SubResponse>(orig).expect("failed to deserialize");
        assert_eq!(expect, result);
    }

    #[test]
    fn test_auth_result_json_deserialize() {
        let test_time = Utc::now().trunc_subsecs(0);
        let orig = json!({
            "i": "test_client_id",
            "T": test_time.timestamp() * 1000
        });
        let expect = AuthResult {
            id: "test_client_id".into(),
            time: test_time,
        };

        let result = serde_json::from_value::<AuthResult>(orig).expect("failed to deserialize");
        assert_eq!(expect, result);
    }

    #[test]
    fn test_server_push_event_json_deserialize_dispatch() {
        #[allow(overflowing_literals)]
        let orig_list = vec![
            json!({
              "e": "error",
              "E": ["...."],
              "i": "client1",
              "T": 123456789
            }),
            json!({
              "e": "subscribed",
              "s": [
                {"channel": "book", "market": "btctwd", "depth": 1},
                {"channel": "trade", "market": "btctwd"}
              ],
              "i": "client1",
              "T": 123456789
            }),
            json!({
              "e": "unsubscribed",
              "s": [
                {"channel": "book", "market": "btctwd", "depth": 1},
                {"channel": "trade", "market": "btctwd"}
              ],
              "i": "client1",
              "T": 123456789
            }),
            json!({
              "e": "authenticated",
              "i": "client-id",
              "T": 1637998469525
            }),
            json!({
             "c": "book",
             "e": "snapshot",
             "M": "btcusdt",
             "a": [["5337.3", "0.1"]],
             "b": [["5333.3", "0.5"]],
             "T": 1637998469525
            }),
            json!({
              "c": "trade",
              "e": "update",
              "M": "btctwd",
              "t":[{
                "p": "5337.3",
                "v": "0.1",
                "T": 123456789,
                "tr": "up"
              }],
              "T": 123456789
            }),
            json!({
             "c": "ticker",
             "e": "snapshot",
             "M": "btctwd",
             "tk": {
                "O": "280007.1",
                "H": "280017.2",
                "L": "280005.3",
                "C": "280004.5",
                "v": "71.01"
             },
             "T": 123456789
            }),
            json!({
              "c": "market_status",
              "e": "update",
              "ms": [{
                  "M": "btctwd",
                  "st": "active",
                  "bu": "btc",
                  "bup": 8,
                  "mba": 0.0004,
                  "qu": "twd",
                  "qup": 1,
                  "mqa": 250,
                  "mws": true
                }],
              "T": 1659428472313
            }),
            json!({
              "c": "user",
              "e": "order_update",
              "o": [{
                 "i": 87,
                 "sd": "bid",
                 "ot": "limit",
                 "p": "21499.0",
                 "sp": "21499.0",
                 "ap": "21499.0",
                 "S": "done",
                 "M": "ethtwd",
                 "T": 1521726960123,
                 "v": "0.2658",
                 "rv": "0.0",
                 "ev": "0.2658",
                 "tc": 1,
                 "ci": "client-oid-1",
                 "gi": 123
              }],
              "T": 1521726960357
            }),
            json!({
              "c": "user",
              "e": "trade_snapshot",
              "t": [{
                "i": 68444,
                "p": "21499.0",
                "v": "0.2658",
                "M": "ethtwd",
                "T": 1521726960357,
                "sd": "bid",
                "f": "3.2",
                "fc": "twd",
                "m": true
              }],
              "T": 1521726960357
            }),
            json!({
              "c": "user",
              "e": "account_update",
              "B": [
                {
                  "cu": "btc",
                  "av": "123.4",
                  "l": "0.5"
                },
                {
                  "cu": "btc",
                  "av": "123.4",
                  "l": "0.5"
                }
              ],
              "T": 123456789,
            }),
        ];

        let mut checked: i8 = 11;
        for (i, orig) in orig_list.into_iter().enumerate() {
            match serde_json::from_value::<ServerPushEvent>(orig)
                .unwrap_or_else(|_| panic!("failed to deserialize at #{}", i))
            {
                ServerPushEvent::Error(_) => {
                    assert_eq!(0, i);
                    checked -= 1
                }
                ServerPushEvent::SubResp(_) => {
                    assert_eq!(1, i);
                    checked -= 1
                }
                ServerPushEvent::UnsubResp(_) => {
                    assert_eq!(2, i);
                    checked -= 1
                }
                ServerPushEvent::AuthResp(_) => {
                    assert_eq!(3, i);
                    checked -= 1
                }
                ServerPushEvent::PubOrderbookFeed(_) => {
                    assert_eq!(4, i);
                    checked -= 1
                }
                ServerPushEvent::PubTradeFeed(_) => {
                    assert_eq!(5, i);
                    checked -= 1
                }
                ServerPushEvent::PubTickerFeed(_) => {
                    assert_eq!(6, i);
                    checked -= 1
                }
                ServerPushEvent::PubMarketStatueFeed(_) => {
                    assert_eq!(7, i);
                    checked -= 1
                }
                ServerPushEvent::PrivOrderbookFeed(_) => {
                    assert_eq!(8, i);
                    checked -= 1
                }
                ServerPushEvent::PrivTradeFeed(_) => {
                    assert_eq!(9, i);
                    checked -= 1
                }
                ServerPushEvent::PrivBalanceFeed(_) => {
                    assert_eq!(10, i);
                    checked -= 1
                }
            }
        }
        assert_eq!(0, checked);
    }
}
