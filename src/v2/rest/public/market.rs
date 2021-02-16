use std::collections::HashMap;

use chrono::serde as chrono_serde;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::common::*;
use crate::v2::rest::api_impl::*;

// ========
// Requests
// ========

/// GET /api/v2/k
///
/// Get OHLC(k line) of a specific market
#[derive(Serialize, Debug)]
pub struct GetOHLC {
    /// Unique market id, check /api/v2/markets for available markets.
    pub market: Symbol,
    /// Returned data points limit, default to 30
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    /// Time period of K line in minute, default to 1
    #[serde(rename = "period")]
    pub period_minutes: u16,
    /// The seconds elapsed since Unix epoch, set to return data after the timestamp only
    #[serde(
        rename = "timestamp",
        skip_serializing_if = "Option::is_none",
        with = "chrono_serde::ts_seconds_option"
    )]
    pub after_timestamp: Option<DateTime>,
}
impl_api!(GetOHLC => Vec<OHLC> : GET, "/api/v2/k");

/// GET /api/v2/depth
///
/// Get depth of a specified market
#[derive(Serialize, Debug)]
pub struct GetDepth {
    /// Unique market id, check /api/v2/markets for available markets.
    pub market: Symbol,
    /// Returned price levels limit, default to maximum value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    /// Sorting by price or by ticker position
    pub sort_by_price: bool,
}
impl_api!(GetDepth => RespDepth : GET, "/api/v2/depth");

/// GET /api/v2/trades
///
/// Get recent trades on market, sorted in reverse creation order.
#[derive(Serialize, Debug)]
pub struct GetPublicTrades {
    /// Unique market id, check /api/v2/markets for available markets.
    pub market: Symbol,
    /// The seconds elapsed since Unix epoch, set to return trades executed before the time only.
    #[serde(rename = "timestamp", with = "chrono_serde::ts_seconds")]
    pub timestamp_before: DateTime,
    /// Trade id, set ot return trades created after the trade.
    #[serde(rename = "from", skip_serializing_if = "Option::is_none")]
    pub after_order_id: Option<u64>,
    /// Trade id, set to return trades created before the trade.
    #[serde(rename = "to", skip_serializing_if = "Option::is_none")]
    pub before_order_id: Option<u64>,
    /// Order the trades by created time, default to 'desc'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<OrderBy>,
    /// Do pagination & return metadata in header (default true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<bool>,
    /// Pagination parameters.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub page_params: Option<PageParams>,
    /// Records to skip, not applied for pagination (default 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
}
impl_api!(GetPublicTrades => Vec<TradeRecord> : GET, "/api/v2/trades");

/// GET /api/v2/markets
///
/// Get all available markets.
#[derive(Serialize, Debug)]
pub struct GetMarkets {}
impl_api!(GetMarkets => Vec<MarketInfo> : GET, "/api/v2/markets");

/// GET /api/v2/summary
///
/// Overview of market data for all tickers.
#[derive(Serialize, Debug)]
pub struct GetMarketsSummary {}
impl_api!(GetMarketsSummary => RespSummary : GET, "/api/v2/summary");

/// GET /api/v2/tickers
///
/// Get ticker of all markets.
#[derive(Serialize, Debug)]
pub struct GetTickers {}
impl_api!(GetTickers => HashMap<Symbol, RespTickerInfo> : GET, "/api/v2/tickers");

/// GET /api/v2/tickers/{path_market}
///
/// Get ticker of specific market.
#[derive(Serialize, Debug)]
pub struct GetTickersOfMarket {
    /// Unique market id, check /api/v2/markets for available markets.
    #[serde(skip)]
    pub market: Symbol,
}
impl_api!(GetTickersOfMarket => RespTickerInfo : GET, dynamic params {
    api_url!(dynamic "/api/v2/tickers/{}", params.market)
});

// =========
// Responses
// =========

/// All Depth of a specified market
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct RespDepth {
    /// timestamp: timestamp
    #[serde(rename = "timestamp", with = "chrono_serde::ts_seconds")]
    pub time: DateTime,
    /// last_update_version: last update version
    pub last_update_version: u64,
    /// last_update_id: last update ID
    pub last_update_id: u64,
    /// asks (list of depth entries): list of asked price/volume
    pub asks: Vec<DepthEntry>,
    /// bids (list of depth entries): list of bid price/volume
    pub bids: Vec<DepthEntry>,
}

/// Overview of market data for all tickers
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
pub struct RespSummary {
    /// tickers: tickers of all markets.
    pub tickers: HashMap<Symbol, RespTickerInfo>,
    /// coins: all coins.
    pub coins: HashMap<String, CoinInfo>,
}

/// Ticker information
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct RespTickerInfo {
    /// at: timestamp in seconds since Unix epoch ,
    #[serde(with = "chrono_serde::ts_seconds")]
    pub at: DateTime,
    /// buy: highest buy price ,
    pub buy: Decimal,
    /// sell: lowest sell price ,
    pub sell: Decimal,
    /// open: price before 24 hours ,
    pub open: Decimal,
    /// low: lowest price within 24 hours ,
    pub low: Decimal,
    /// high: highest price within 24 hours ,
    pub high: Decimal,
    /// last: last traded price ,
    #[serde(rename = "last")]
    pub last_price: Decimal,
    /// vol: traded volume within 24 hours ,
    #[serde(alias = "vol")]
    pub volume: Decimal,
    /// vol_in_btc: traded volume within 24 hours in equal BTC
    #[serde(alias = "vol_in_btc")]
    pub volume_in_btc: Decimal,
}

// ============================
// Inner structures and options
// ============================

/// OHLC in K line
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct OHLC {
    // note: field order matters
    /// timestamp: timestamp
    #[serde(with = "chrono_serde::ts_seconds")]
    pub time: DateTime,

    /// Opening price
    pub open: Decimal,
    /// Highest price
    pub high: Decimal,
    /// Lowest price
    pub low: Decimal,
    /// Closing price
    pub close: Decimal,

    /// volume: total trade volume in given period
    pub volume: Decimal,
}

/// Depth entry of a specified market.
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct DepthEntry {
    /// price: price of given level
    pub price: Decimal,
    /// volume: volume
    pub volume: Decimal,
}

/// Trade record
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct TradeRecord {
    /// id: trade id
    pub id: u64,
    /// price: strike price
    pub price: Option<Decimal>,
    /// volume: traded volume
    pub volume: Option<Decimal>,
    /// funds: total traded amount
    pub funds: Option<Decimal>,
    /// Unique market id, check /api/v2/markets for available markets.
    pub market: Symbol,
    /// market_name: market name
    pub market_name: String,
    /// created_at_in_ms: created timestamp (millisecond)
    #[serde(with = "chrono_serde::ts_seconds")]
    pub created_at: DateTime,
    #[serde(with = "chrono_serde::ts_milliseconds")]
    pub created_at_in_ms: DateTime,
    /// side: 'bid' or 'ask'; side of maker for public trades; side of your order when querying your own trades (can be 'self-trade')
    pub side: TradeSide,
    /// fee: your related fee (show ask side if self-trade)
    pub fee: Option<Decimal>,
    /// fee_currency: fee currency (show ask side if self-trade)
    pub fee_currency: Option<String>,
    /// order_id: order related to you (show ask side if self-trade)
    pub order_id: Option<u64>,
    /// info: provide ask/bid info for order owner
    #[serde(default)]
    pub info: Option<TradeMakerType>,
}

/// Trade info inside trade record
#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(tag = "maker", rename_all = "lowercase")]
pub enum TradeMakerType {
    Ask { ask: TradeMakerInfo },
    Bid { bid: TradeMakerInfo },
    Unknown,
}

impl TradeMakerType {
    pub fn is_unknown(&self) -> bool {
        self == &Self::Unknown
    }
}

impl Default for TradeMakerType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Trade info inside trade record
#[derive(Deserialize, Default, Eq, PartialEq, Debug)]
pub struct TradeMakerInfo {
    /// fee: trade fee
    pub fee: Decimal,
    /// fee_currency: currency of trade fee
    pub fee_currency: String,
    /// order_id: order ID
    pub order_id: u64,
}

/// Market information
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
#[serde(default)]
pub struct MarketInfo {
    /// id: unique market id, check /api/v2/markets for available markets.
    pub id: Symbol,
    /// name: market name.
    pub name: String,
    /// base_unit: base unit.
    pub base_unit: String,
    /// base_unit_precision: fixed precision of base unit.
    pub base_unit_precision: u8,
    /// min_base_amount: minimum of base amount.
    pub min_base_amount: Decimal,
    /// quote_unit: quote unit.
    pub quote_unit: String,
    /// quote_unit_precision: fixed precision of quote unit.
    pub quote_unit_precision: u8,
    /// min_quote_amount: minimum of quote amount.
    pub min_quote_amount: Decimal,
}

/// Coin information
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct CoinInfo {
    /// name: coin name.
    pub name: String,
    /// withdraw: able to withdraw.
    #[serde(deserialize_with = "crate::util::serde::bool_from_onoff")]
    pub withdraw: bool,
    /// deposit: able to deposit.
    #[serde(deserialize_with = "crate::util::serde::bool_from_onoff")]
    pub deposit: bool,
    /// trade: able to trade.
    #[serde(deserialize_with = "crate::util::serde::bool_from_onoff")]
    pub trade: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test_util::*;
    use chrono::{TimeZone, Utc};
    use rust_decimal_macros::dec;
    use surf::Client as HTTPClient;
    use surf_vcr::VcrMode;

    async fn create_client(cassette: &'static str) -> HTTPClient {
        let mut path_builder = test_resource_path();
        path_builder.push("rest");
        path_builder.push("public");
        path_builder.push("market");
        path_builder.push(cassette);
        create_test_recording_client(VcrMode::Replay, path_builder.as_path().to_str().unwrap())
            .await
    }

    #[async_std::test]
    async fn get_ohlc() {
        let params = GetOHLC {
            market: "btctwd".into(),
            limit: Some(10),
            period_minutes: 1,
            after_timestamp: None,
        };
        let resp = create_client("get_ohlc.yaml")
            .await
            .send(params.to_request())
            .await
            .expect("Error while sending request");
        let result = GetOHLC::read_response(resp.into()).await;
        let ohlcs: Vec<OHLC> = result.expect("failed to parse result");
        assert_eq!(ohlcs.len(), 10);
        assert_eq!(
            ohlcs[1],
            OHLC {
                time: Utc.timestamp(1636257660, 0),
                open: dec!(1735077.9),
                high: dec!(1735077.9),
                low: dec!(1735077.9),
                close: dec!(1735077.9),
                volume: dec!(0.0778),
            }
        );
        assert_eq!(
            ohlcs[3],
            OHLC {
                time: Utc.timestamp(1636257780, 0),
                open: dec!(1738000),
                high: dec!(1738000),
                low: dec!(1738000),
                close: dec!(1738000),
                volume: dec!(0),
            }
        );
    }

    #[async_std::test]
    async fn get_depth() {
        let params = GetDepth {
            market: "btctwd".into(),
            limit: Some(10),
            sort_by_price: true,
        };
        let resp = create_client("get_depth.yaml")
            .await
            .send(params.to_request())
            .await
            .expect("Error while sending request");
        let result = GetDepth::read_response(resp.into()).await;
        let depth_info: RespDepth = result.expect("failed to parse result");

        assert_eq!(depth_info.asks.len(), 10);
        assert_eq!(
            depth_info.asks[9],
            DepthEntry {
                price: dec!(1738000.0),
                volume: dec!(0.1159757),
            }
        );

        assert_eq!(depth_info.bids.len(), 10);
        assert_eq!(
            depth_info.bids[8],
            DepthEntry {
                price: dec!(1732000.0),
                volume: dec!(0.05773672),
            }
        );
    }

    #[async_std::test]
    async fn get_public_trades() {
        let params = GetPublicTrades {
            market: "btctwd".into(),
            timestamp_before: Utc.timestamp(1636212254, 0),
            after_order_id: None,
            before_order_id: None,
            order_by: None,
            pagination: None,
            page_params: None,
            offset: None,
        };
        let resp = create_client("get_public_trades.yaml")
            .await
            .send(params.to_request())
            .await
            .expect("Error while sending request");
        let result = GetPublicTrades::read_response(resp.into()).await;
        let trade_list: Vec<TradeRecord> = result.expect("failed to parse result");
        assert_eq!(trade_list.len(), 50);
        assert_eq!(
            trade_list[5],
            TradeRecord {
                id: 29219425,
                price: Some(dec!(1699352.1)),
                volume: Some(dec!(0.001092)),
                funds: Some(dec!(1855.7)),
                market: "btctwd".to_string(),
                market_name: "BTC/TWD".to_string(),
                created_at: Utc.timestamp(1636212047, 0),
                created_at_in_ms: Utc.timestamp(1636212047, 217000000),
                side: TradeSide::Ask,
                fee: None,
                fee_currency: None,
                order_id: None,
                info: None,
            }
        );
    }

    #[async_std::test]
    async fn get_markets() {
        let params = GetMarkets {};
        let resp = create_client("get_markets.yaml")
            .await
            .send(params.to_request())
            .await
            .expect("Error while sending request");
        let result = GetMarkets::read_response(resp.into()).await;
        let market_list: Vec<MarketInfo> = result.expect("failed to parse result");
        assert_eq!(market_list.len(), 34);
        assert_eq!(
            market_list[0],
            MarketInfo {
                id: "maxtwd".into(),
                name: "MAX/TWD".into(),
                base_unit: "max".into(),
                base_unit_precision: 2,
                min_base_amount: dec!(21),
                quote_unit: "twd".into(),
                quote_unit_precision: 4,
                min_quote_amount: dec!(250),
            }
        )
    }

    #[async_std::test]
    async fn get_summary() {
        let params = GetMarketsSummary {};
        let resp = create_client("get_summary.yaml")
            .await
            .send(params.to_request())
            .await
            .expect("Error while sending request");
        let result = GetMarketsSummary::read_response(resp.into()).await;
        let summary: RespSummary = result.expect("failed to parse result");

        assert_eq!(summary.coins.len(), 19);
        assert_eq!(
            summary.coins.get("max"),
            Some(&CoinInfo {
                name: "max".into(),
                withdraw: true,
                deposit: true,
                trade: true,
            })
        );

        assert_eq!(summary.tickers.len(), 34);
        assert_eq!(
            summary.tickers.get("btctwd"),
            Some(&RespTickerInfo {
                at: Utc.timestamp(1636258205, 0),
                buy: dec!(1737000.0),
                sell: dec!(1738000.0),
                open: dec!(1708337.2),
                low: dec!(1682500.0),
                high: dec!(1739517.2),
                last_price: dec!(1738000.0),
                volume: dec!(23.70350862),
                volume_in_btc: dec!(23.70350862),
            })
        );
    }

    #[async_std::test]
    async fn get_tickers() {
        let params = GetTickers {};
        let resp = create_client("get_tickers.yaml")
            .await
            .send(params.to_request())
            .await
            .expect("Error while sending request");
        let result = GetTickers::read_response(resp.into()).await;
        let tickers: HashMap<Symbol, RespTickerInfo> = result.expect("failed to parse result");
        assert_eq!(tickers.len(), 34);
        assert_eq!(
            tickers.get("maxtwd"),
            Some(&RespTickerInfo {
                at: Utc.timestamp(1636258205, 0),
                buy: dec!(11.4951),
                sell: dec!(11.5376),
                open: dec!(11.5499),
                low: dec!(11.4812),
                high: dec!(11.5499),
                last_price: dec!(11.5377),
                volume: dec!(78450.18),
                volume_in_btc: dec!(0.51921291849962826),
            })
        )
    }

    #[async_std::test]
    async fn get_ticker_of_market() {
        let params = GetTickersOfMarket {
            market: "btctwd".into(),
        };
        let resp = create_client("get_ticker_of_market.yaml")
            .await
            .send(params.to_request())
            .await
            .expect("Error while sending request");
        let result = GetTickersOfMarket::read_response(resp.into()).await;
        let ticker: RespTickerInfo = result.expect("failed to parse result");
        assert_eq!(
            ticker,
            RespTickerInfo {
                at: Utc.timestamp(1636258205, 0),
                buy: dec!(1737000.0),
                sell: dec!(1738000.0),
                open: dec!(1708337.2),
                low: dec!(1682500.0),
                high: dec!(1739517.2),
                last_price: dec!(1738000.0),
                volume: dec!(23.70350862),
                volume_in_btc: dec!(23.70350862),
            }
        );
    }
}
