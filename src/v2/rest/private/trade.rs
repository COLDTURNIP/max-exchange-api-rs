use chrono::serde as chrono_serde;
use serde::Serialize;

use crate::common::*;
use crate::v2::rest::api_impl::*;

// ========
// Requests
// ========

/// GET /api/v2/trades/my/of_order
///
/// Get your executed trades related to a order.
#[derive(Serialize, Debug)]
pub struct GetMyTradesOfOrder {
    /// Unique order id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    /// User specific order id. maximum length of client_oid must less or equal to 36. persistence, server will validate uniqueness within 24 hours only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_oid: Option<String>,
}
impl_api!(GetMyTradesOfOrder => Vec<TradeRecord> : auth GET, "/api/v2/trades/my/of_order");

/// GET /api/v2/trades/my
///
/// Get your executed trades, sorted in reverse creation order.
#[derive(Serialize, Debug)]
pub struct GetMyTrades {
    /// Unique market id, check /api/v2/markets for available markets.
    pub market: Symbol,
    #[serde(
        rename = "timestamp",
        skip_serializing_if = "Option::is_none",
        with = "chrono_serde::ts_seconds_option"
    )]
    /// The seconds elapsed since Unix epoch, set to return trades executed before the time only.
    pub timestamp_before: Option<DateTime>,
    /// Trade id, set ot return trades created after the trade.
    #[serde(rename = "from", skip_serializing_if = "Option::is_none")]
    pub after_order_id: Option<u64>,
    /// Trade id, set to return trades created before the trade.
    #[serde(rename = "to", skip_serializing_if = "Option::is_none")]
    pub before_order_id: Option<u64>,
    /// Order the trades by created time, default to `'desc'`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<OrderBy>,
    /// Do pagination & return metadata in header (default `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<bool>,
    /// Pagination parameters, see [`crate::common::PageParams`].
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub page_params: Option<PageParams>,
    /// Records to skip, not applied for pagination (default `0`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
}
impl_api!(GetMyTrades => Vec<TradeRecord> : auth GET, "/api/v2/trades/my");

// =========
// Responses
// =========

// Nothing here. All responses are warped in vectors.

// ============================
// Inner structures and options
// ============================

pub use crate::v2::rest::public::{TradeMakerInfo, TradeMakerType, TradeRecord};

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
        path_builder.push("private");
        path_builder.push("trade");
        path_builder.push(cassette);
        create_test_recording_client(VcrMode::Replay, path_builder.as_path().to_str().unwrap())
            .await
    }

    #[async_std::test]
    async fn get_single_trade() {
        let params = GetMyTradesOfOrder {
            id: Some(1545763894),
            client_oid: None,
        };
        let resp = create_client("get_single_trade.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result: Vec<TradeRecord> = GetMyTradesOfOrder::read_response(resp.into())
            .await
            .unwrap();
        assert_eq!(
            result,
            vec![TradeRecord {
                id: 29009013,
                price: Some(dec!(52.0)),
                volume: Some(dec!(3.14)),
                funds: Some(dec!(163.28)),
                market: "dotusdt".into(),
                market_name: "DOT/USDT".into(),
                created_at: Utc.timestamp(1635853634, 0),
                created_at_in_ms: Utc.timestamp(1635853634, 52000000),
                side: TradeSide::Bid,
                fee: Some(dec!(0.08908907)),
                fee_currency: Some("max".into()),
                order_id: None,
                info: None
            }]
        );
    }

    #[async_std::test]
    async fn get_all_trades() {
        let params = GetMyTrades {
            market: "dotusdt".into(),
            timestamp_before: Some(Utc.timestamp(1635854000, 0)),
            after_order_id: Some(29009000),
            before_order_id: None,
            order_by: None,
            pagination: None,
            page_params: None,
            offset: None,
        };
        let resp = create_client("get_all_trades.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result: Vec<TradeRecord> = GetMyTrades::read_response(resp.into()).await.unwrap();
        assert_eq!(
            result,
            vec![TradeRecord {
                id: 29009013,
                price: Some(dec!(52.0)),
                volume: Some(dec!(3.14)),
                funds: Some(dec!(163.28)),
                market: "dotusdt".into(),
                market_name: "DOT/USDT".into(),
                created_at: Utc.timestamp(1635853634, 0),
                created_at_in_ms: Utc.timestamp(1635853634, 52000000),
                side: TradeSide::Bid,
                fee: Some(dec!(0.08908907)),
                fee_currency: Some("max".into()),
                order_id: Some(1545763894),
                info: Some(TradeMakerType::Bid {
                    bid: TradeMakerInfo {
                        fee: dec!(0.08908907),
                        fee_currency: "max".into(),
                        order_id: 1545763894,
                    }
                }),
            }]
        );
    }
}
