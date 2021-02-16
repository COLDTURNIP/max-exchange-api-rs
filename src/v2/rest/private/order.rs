use chrono::serde as chrono_serde;
use http_types::Request as HTTPRequest;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::common::*;
use crate::v2::rest::api_impl::*;
use crate::v2::rest::internal;

// ========
// Requests
// ========

/// GET /api/v2/order
///
/// Get a specific order.
#[derive(Serialize, Default, Debug)]
pub struct GetOrder {
    /// Unique order id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    /// User specific order id. Maximum length of client_oid must less or equal to 36. persistence, server will validate uniqueness within 24 hours only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_oid: Option<String>,
}
impl_api!(GetOrder => RespOrder : auth GET, "/api/v2/order");

/// GET /api/v2/orders
///
/// Get your orders, results is paginated.
#[derive(Serialize, Debug)]
pub struct GetOrders {
    /// Unique market id, check /api/v2/markets for available markets.
    pub market: Symbol,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    /// Filter by states, default to `['wait', 'convert']`.
    pub state: Vec<OrderState>,
    /// Order in created time, default to `'asc'`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<OrderBy>,
    /// Group order id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u64>,
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

impl internal::RestApiBase for GetOrders {
    endpoint_binding!(fixed "/api/v2/orders");
    type Response = Vec<RespOrder>;
}

impl GetOrders {
    convert_from_response!(Vec<RespOrder>);

    pub fn to_request(&self, credentials: &crate::Credentials) -> HTTPRequest {
        let (url, header_payload, header_signature) = {
            use internal::RestApiBase;

            let mut url = self.get_url();
            let path = url.path().to_string();
            let params = internal::AuthParamsOuterWrapper {
                path: &path,
                inner: internal::AuthParamsInnerWrapper {
                    params: self,
                    nonce: credentials.nonce(),
                },
            };
            {
                // workaround for "state[]=..."
                let mut qs_builder = url.query_pairs_mut();
                qs_builder.append_pair("market", &self.market);
                self.state.iter().for_each(|item| {
                    qs_builder.append_pair("state[]", item.as_srt());
                });
                if let Some(ref order_by) = self.order_by {
                    qs_builder.append_pair(
                        "order_by",
                        format!("{:?}", order_by).to_lowercase().as_str(),
                    );
                }
                if let Some(ref pagination) = self.pagination {
                    qs_builder.append_pair("pagination", &pagination.to_string());
                }
                if let Some(ref page_params) = self.page_params {
                    qs_builder.append_pair("page", &page_params.page.to_string());
                    qs_builder.append_pair("limit", &page_params.limit.to_string());
                }
                if let Some(ref offset) = self.offset {
                    qs_builder.append_pair("offset", &offset.to_string());
                }
                qs_builder.append_pair("nonce", &params.inner.nonce.to_string());
            }
            let (payload, signature) = params.signed_payload(credentials);
            (url, payload, signature)
        };

        let mut req = HTTPRequest::get(url);
        req.insert_header(internal::HEADER_AUTH_ACCESS_KEY, &credentials.access_key);
        req.insert_header(internal::HEADER_AUTH_PAYLOAD, header_payload);
        req.insert_header(internal::HEADER_AUTH_SIGNATURE, header_signature);
        req
    }
}

/// POST /api/v2/orders
///
/// Create a sell/buy order.
#[derive(Serialize, Debug)]
pub struct CreateOrder {
    /// Create a sell/buy order.
    pub market: Symbol,
    /// `'sell'` or `'buy'`.
    pub side: OrderSide,
    /// Total amount to sell/buy, an order could be partially executed.
    pub volume: Decimal,
    /// Price of a unit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    /// User specific order id. maximum length of client_oid must less or equal to 36. persistence, server will validate uniqueness within 24 hours only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_oid: Option<String>,
    /// Price to trigger a stop order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Decimal>,
    /// `'limit'`, `'market'`, `'stop_limit'`, `'stop_market'`, `'post_only'` or `'ioc_limit'`.
    pub ord_type: OrderType,
    /// Group order id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u64>,
}
impl_api!(CreateOrder => RespOrder : auth POST, "/api/v2/orders");

// TODO: implement batch order creation
// impl_api!(CreateOneByOneOrder => POST "/api/v2/orders/multi/onebyone")

/// POST /api/v2/order/delete
///
/// Cancel an order.
#[derive(Serialize, Debug)]
pub struct DeleteOrder {
    /// Unique order id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    /// User specific order id. maximum length of client_oid must less or equal to 36. persistence, server will validate uniqueness within 24 hours only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_oid: Option<String>,
}
impl_api!(DeleteOrder => RespOrder : auth POST, "/api/v2/order/delete");

/// POST /api/v2/orders/clear
///
/// Cancel all your orders with given market and side.
#[derive(Serialize, Debug)]
pub struct ClearOrders {
    /// Unique market id, check /api/v2/markets for available markets.
    pub market: Symbol,
    /// Set tp cancel only sell (asks) or buy (bids) orders.
    pub side: OrderSide,
    /// Group order id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u64>,
}
impl_api!(ClearOrders => Vec<RespOrder> : auth POST, "/api/v2/orders/clear");

// =========
// Responses
// =========

/// Submitted order detail.
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
#[serde(default)]
pub struct RespOrder {
    /// id (integer, optional): unique order id.
    pub id: Option<u64>,
    /// client_oid (string, optional): user specific order id. maximum length of client_oid must less or equal to 36. persistence, server will validate uniqueness within 24 hours only.
    pub client_oid: Option<String>,
    /// side (string, optional): `'sell'` or `'buy'`
    pub side: OrderSide,
    /// ord_type (string, optional): `'limit'`, `'market'`, `'stop_limit'`, `'stop_market'`, `'post_only'` or `'ioc_limit'`
    pub ord_type: OrderType,
    /// price (string, optional): price of a unit.
    pub price: Option<Decimal>,
    /// stop_price (string, optional): price to trigger a stop order.
    pub stop_price: Option<Decimal>,
    /// avg_price (string, optional): average execution price.
    pub avg_price: Option<Decimal>,
    /// state (string, optional): `'wait'`, `'done'`, `'cancel'`, or `'convert'`; `'wait'` means waiting for fulfillment; `'done'` means fullfilled; `'cancel'` means cancelled; `'convert'` means the stop order is triggered.
    pub state: OrderState,
    /// market (string, optional): market id, check /api/v2/markets for available markets.
    pub market: Symbol,
    /// created_at (integer, optional): created timestamp (second).
    #[serde(with = "chrono_serde::ts_seconds_option")]
    pub created_at: Option<DateTime>,
    /// created_at_in_ms (integer, optional): created timestamp (millisecond).
    #[serde(with = "chrono_serde::ts_milliseconds_option")]
    pub created_at_in_ms: Option<DateTime>,
    /// updated_at (integer, optional): updated timestamp (second).
    #[serde(with = "chrono_serde::ts_seconds_option")]
    pub updated_at: Option<DateTime>,
    /// updated_at_in_ms (integer, optional): updated timestamp (millisecond).
    #[serde(with = "chrono_serde::ts_milliseconds_option")]
    pub updated_at_in_ms: Option<DateTime>,
    /// volume (string, optional): total amount to sell/buy, an order could be partially executed.
    pub volume: Option<Decimal>,
    /// remaining_volume (string, optional): remaining volume.
    pub remaining_volume: Option<Decimal>,
    /// executed_volume (string, optional): executed volume.
    pub executed_volume: Option<Decimal>,
    /// trades_count (integer, optional): trade count.
    pub trades_count: Option<u64>,
    /// group_id (integer, optional): group order id.
    pub group_id: Option<u64>,
}

// ============================
// Inner structures and options
// ============================

/// Order types.
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Limit,
    Market,
    StopLimit,
    StopMarket,
    PostOnly,
    IocLimit,
    Unknown,
}

impl OrderType {
    pub fn is_unknown(&self) -> bool {
        self == &Self::Unknown
    }
}

impl Default for OrderType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Order state.
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum OrderState {
    Wait,
    Done,
    Cancel,
    Convert,
    Finalizing,
    Failed,
    Unknown,
}

impl OrderState {
    pub fn is_wait(&self) -> bool {
        self == &Self::Wait
    }
    pub fn is_done(&self) -> bool {
        self == &Self::Done
    }
    pub fn is_cancel(&self) -> bool {
        self == &Self::Cancel
    }
    pub fn is_convert(&self) -> bool {
        self == &Self::Convert
    }
    pub fn is_unknown(&self) -> bool {
        self == &Self::Unknown
    }

    pub fn as_srt(&self) -> &'static str {
        match *self {
            Self::Wait => "wait",
            Self::Done => "done",
            Self::Cancel => "cancel",
            Self::Convert => "convert",
            Self::Finalizing => "finalizing",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }
}

impl Default for OrderState {
    fn default() -> Self {
        Self::Unknown
    }
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
        path_builder.push("private");
        path_builder.push("order");
        path_builder.push(cassette);
        create_test_recording_client(VcrMode::Replay, path_builder.as_path().to_str().unwrap())
            .await
    }

    #[async_std::test]
    async fn get_single_order() {
        let params = GetOrder {
            id: Some(1545763894),
            client_oid: None,
        };
        let resp = create_client("get_single_order.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result: RespOrder = GetOrder::read_response(resp.into()).await.unwrap();
        assert_eq!(
            result,
            RespOrder {
                id: Some(1545763894),
                client_oid: None,
                side: OrderSide::Buy,
                ord_type: OrderType::Limit,
                price: Some(dec!(52.0)),
                stop_price: None,
                avg_price: Some(dec!(52.0)),
                state: OrderState::Done,
                market: "dotusdt".into(),
                created_at: Some(Utc.timestamp(1635853116, 0)),
                created_at_in_ms: Some(Utc.timestamp(1635853116, 171000000)),
                updated_at: Some(Utc.timestamp(1635853634, 0)),
                updated_at_in_ms: Some(Utc.timestamp(1635853634, 47000000)),
                volume: Some(dec!(3.14)),
                remaining_volume: Some(dec!(0.0)),
                executed_volume: Some(dec!(3.14)),
                trades_count: Some(1),
                group_id: None
            }
        );
    }

    #[async_std::test]
    async fn get_all_orders() {
        let params = GetOrders {
            market: "dotusdt".into(),
            state: vec![
                OrderState::Wait,
                OrderState::Done,
                OrderState::Cancel,
                OrderState::Convert,
                OrderState::Finalizing,
                OrderState::Failed,
            ],
            order_by: None,
            group_id: None,
            pagination: None,
            page_params: None,
            offset: None,
        };
        let resp = create_client("get_all_orders.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result: Vec<RespOrder> = GetOrders::read_response(resp.into()).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[1],
            RespOrder {
                id: Some(1545763894),
                client_oid: None,
                side: OrderSide::Buy,
                ord_type: OrderType::Limit,
                price: Some(dec!(52.0)),
                stop_price: None,
                avg_price: Some(dec!(52.0)),
                state: OrderState::Done,
                market: "dotusdt".into(),
                created_at: Some(Utc.timestamp(1635853116, 0)),
                created_at_in_ms: Some(Utc.timestamp(1635853116, 171000000)),
                updated_at: Some(Utc.timestamp(1635853634, 0)),
                updated_at_in_ms: Some(Utc.timestamp(1635853634, 47000000)),
                volume: Some(dec!(3.14)),
                remaining_volume: Some(dec!(0.0)),
                executed_volume: Some(dec!(3.14)),
                trades_count: Some(1),
                group_id: None
            }
        );
    }

    #[async_std::test]
    async fn create_order() {
        let params = CreateOrder {
            market: "maxusdt".into(),
            side: OrderSide::Sell,
            volume: dec!(23.4),
            price: Some(dec!(1.0)),
            client_oid: Some("max_rs_api_case_create_order".into()),
            stop_price: None,
            ord_type: OrderType::Limit,
            group_id: None,
        };
        let resp = create_client("create_order.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result: RespOrder = CreateOrder::read_response(resp.into()).await.unwrap();
        assert_eq!(
            result,
            RespOrder {
                id: Some(1601376421),
                client_oid: Some("(test erased client_oid)".into()),
                side: OrderSide::Sell,
                ord_type: OrderType::Limit,
                price: Some(dec!(1.0)),
                stop_price: None,
                avg_price: Some(dec!(0.0)),
                state: OrderState::Wait,
                market: "maxusdt".into(),
                created_at: Some(Utc.timestamp(1636876252, 0)),
                created_at_in_ms: Some(Utc.timestamp(1636876252, 685000000)),
                updated_at: Some(Utc.timestamp(1636876252, 0)),
                updated_at_in_ms: Some(Utc.timestamp(1636876252, 685000000)),
                volume: Some(dec!(23.4)),
                remaining_volume: Some(dec!(23.4)),
                executed_volume: Some(dec!(0.0)),
                trades_count: Some(0),
                group_id: None
            }
        );
    }

    #[async_std::test]
    async fn delete_order() {
        let params = DeleteOrder {
            id: Some(1545763894),
            client_oid: None,
        };
        let resp = create_client("delete_order.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result: RespOrder = DeleteOrder::read_response(resp.into()).await.unwrap();
        assert_eq!(
            result,
            RespOrder {
                id: Some(1545763894),
                client_oid: None,
                side: OrderSide::Buy,
                ord_type: OrderType::Limit,
                price: Some(dec!(52.0)),
                stop_price: None,
                avg_price: Some(dec!(52.0)),
                state: OrderState::Done,
                market: "dotusdt".into(),
                created_at: Some(Utc.timestamp(1635853116, 0)),
                created_at_in_ms: Some(Utc.timestamp(1635853116, 171000000)),
                updated_at: Some(Utc.timestamp(1635853634, 0)),
                updated_at_in_ms: Some(Utc.timestamp(1635853634, 47000000)),
                volume: Some(dec!(3.14)),
                remaining_volume: Some(dec!(0.0)),
                executed_volume: Some(dec!(3.14)),
                trades_count: Some(1),
                group_id: None
            }
        );
    }

    #[async_std::test]
    async fn clear_order() {
        let params = ClearOrders {
            market: "maxusdt".into(),
            side: OrderSide::Sell,
            group_id: None,
        };
        let resp = create_client("clear_order.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result: Vec<RespOrder> = ClearOrders::read_response(resp.into()).await.unwrap();
        assert_eq!(
            result,
            vec![RespOrder {
                id: Some(1601361566),
                client_oid: Some("(test erased client_oid)".into()),
                side: OrderSide::Sell,
                ord_type: OrderType::Limit,
                price: Some(dec!(1.0)),
                stop_price: None,
                avg_price: Some(dec!(0.0)),
                state: OrderState::Wait,
                market: "maxusdt".into(),
                created_at: Some(Utc.timestamp(1636875985, 0)),
                created_at_in_ms: Some(Utc.timestamp(1636875985, 861000000)),
                updated_at: Some(Utc.timestamp(1636875985, 0)),
                updated_at_in_ms: Some(Utc.timestamp(1636875985, 861000000)),
                volume: Some(dec!(23.4)),
                remaining_volume: Some(dec!(23.4)),
                executed_volume: Some(dec!(0.0)),
                trades_count: Some(0),
                group_id: None
            }]
        );
    }
}
