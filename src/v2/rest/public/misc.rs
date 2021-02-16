use std::convert::From;

use chrono::{NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::common::*;
use crate::v2::rest::api_impl::*;

// ========
// Requests
// ========

/// GET /api/v2/vip_levels
///
/// Get all VIP level fees.
#[derive(Serialize, Debug)]
pub struct GetVIPLevels {}
impl_api!(GetVIPLevels => Vec<RespVIPLevel> : GET, "/api/v2/vip_levels");

/// GET /api/v2/vip_levels/{level}
///
/// Get VIP level fee by level.
#[derive(Serialize, Debug)]
pub struct GetVIPByLevel {
    /// VIP level
    #[serde(skip)]
    pub level: u8,
}
impl_api!(GetVIPByLevel => RespVIPLevel : GET, dynamic params {
    api_url!(dynamic "/api/v2/vip_levels/{}", params.level)
});

/// GET /api/v2/currencies
///
/// Get all available currencies.
#[derive(Serialize, Debug)]
pub struct GetCurrencies {}
impl_api!(GetCurrencies => Vec<CurrencyInfo> : GET, "/api/v2/currencies");

/// GET /api/v2/timestamp
///
/// Get server current time, in seconds since Unix epoch
#[derive(Serialize, Debug)]
pub struct GetTimestamp {}
impl_api!(GetTimestamp => RespTimestamp : GET, "/api/v2/timestamp");

/// GET /api/v2/withdrawal/constraint
///
/// Withdrawal constraints
#[derive(Serialize, Debug)]
pub struct GetWithdrawalConstraints {
    /// Unique currency id, check /api/v2/currencies for available currencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}
impl_api!(GetWithdrawalConstraints => Vec<WithdrawalConstraints> : GET, "/api/v2/withdrawal/constraint");

// =========
// Responses
// =========

/// Response of GET /api/v2/vip_levels*
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
#[serde(default)]
pub struct RespVIPLevel {
    /// level: VIP level
    pub level: u8,
    /// minimum_trading_volume: minimun trading volume for this level
    pub minimum_trading_volume: Decimal,
    /// minimum_staking_volume: minimun staking volume for this level
    pub minimum_staking_volume: Decimal,
    /// maker_fee: current maker fee
    pub maker_fee: Decimal,
    /// taker_fee: current taker fee
    pub taker_fee: Decimal,
}

/// Server current time, in seconds since Unix epoch.
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct RespTimestamp(pub i64);

impl From<RespTimestamp> for DateTime {
    fn from(resp: RespTimestamp) -> Self {
        DateTime::from_utc(NaiveDateTime::from_timestamp(resp.0, 0), Utc)
    }
}

// ============================
// Inner structures and options
// ============================

/// Response of GET /api/v2/currencies
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
#[serde(default)]
pub struct CurrencyInfo {
    /// id: unique currency id
    pub id: String,
    /// precision: fixed precision of the currency
    pub precision: u8,
    /// sygna_supported: if support sygna travel rule
    pub sygna_supported: bool,
}

/// Response of GET /api/v2/withdrawal/constraint
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
#[serde(default)]
pub struct WithdrawalConstraints {
    /// currency: currency id.
    pub currency: String,
    /// fee: withdraw fee.
    pub fee: Decimal,
    /// ratio: withdraw fee ratio.
    pub ratio: Decimal,
    /// min_amount: minimum withdrawal amount.
    pub min_amount: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test_util::*;
    use chrono::TimeZone;
    use rust_decimal_macros::dec;
    use surf::Client as HTTPClient;
    use surf_vcr::VcrMode;

    async fn create_client(cassette: &'static str) -> HTTPClient {
        let mut path_builder = test_resource_path();
        path_builder.push("rest");
        path_builder.push("public");
        path_builder.push("misc");
        path_builder.push(cassette);
        create_test_recording_client(VcrMode::Replay, path_builder.as_path().to_str().unwrap())
            .await
    }

    #[async_std::test]
    async fn get_vip_level_list() {
        let params = GetVIPLevels {};
        let resp = create_client("get_vip_level_list.yaml")
            .await
            .send(params.to_request())
            .await
            .expect("Error while sending request");
        let result = GetVIPLevels::read_response(resp.into()).await;
        let levels: Vec<RespVIPLevel> = result.expect("failed to parse result");
        for lv in 0..10 {
            assert_eq!(levels[lv].level, lv as u8);
        }
        assert_eq!(
            levels[4],
            RespVIPLevel {
                level: 4,
                minimum_trading_volume: dec!(150000000),
                minimum_staking_volume: dec!(10000),
                maker_fee: dec!(0),
                taker_fee: dec!(0.0009),
            }
        )
    }

    #[async_std::test]
    async fn get_vip_by_level() {
        let params = GetVIPByLevel { level: 3 };
        let resp = create_client("get_vip_by_level.yaml")
            .await
            .send(params.to_request())
            .await
            .expect("Error while sending request");
        let result = GetVIPByLevel::read_response(resp.into()).await;
        let level: RespVIPLevel = result.expect("failed to parse result");
        assert_eq!(
            level,
            RespVIPLevel {
                level: 3,
                minimum_trading_volume: dec!(30000000),
                minimum_staking_volume: dec!(10000),
                maker_fee: dec!(0),
                taker_fee: dec!(0.00105),
            }
        );
    }

    #[async_std::test]
    async fn get_currencies() {
        let params = GetCurrencies {};
        let resp = create_client("get_currencies.yaml")
            .await
            .send(params.to_request())
            .await
            .expect("Error while sending request");
        let result = GetCurrencies::read_response(resp.into()).await;
        let currencies: Vec<CurrencyInfo> = result.expect("failed to parse result");
        assert_eq!(
            currencies[0],
            CurrencyInfo {
                id: "twd".into(),
                precision: 0,
                sygna_supported: false
            }
        );
    }

    #[async_std::test]
    async fn get_timestamp() {
        let params = GetTimestamp {};
        let resp = create_client("get_timestamp.yaml")
            .await
            .send(params.to_request())
            .await
            .expect("Error while sending request");
        let result = GetTimestamp::read_response(resp.into()).await;
        let ts: RespTimestamp = result.expect("failed to parse result");
        assert_eq!(ts.0, 1636258261);
        assert_eq!(Into::<DateTime>::into(ts), Utc.timestamp(1636258261, 0))
    }

    #[async_std::test]
    async fn get_withdrawal_constraints() {
        let client = create_client("get_withdrawal_constraints.yaml").await;

        let params_all = GetWithdrawalConstraints { currency: None };
        let resp = client
            .send(params_all.to_request())
            .await
            .expect("Error while sending request");
        let result = GetWithdrawalConstraints::read_response(resp.into()).await;
        let constrains_all: Vec<WithdrawalConstraints> = result.expect("failed to parse result");
        assert_eq!(constrains_all.len(), 31);

        let params_single = GetWithdrawalConstraints {
            currency: Some("twd".into()),
        };
        let resp = client
            .send(params_single.to_request())
            .await
            .expect("Error while sending request");
        let result = GetWithdrawalConstraints::read_response(resp.into()).await;
        let mut constrains_single: Vec<WithdrawalConstraints> =
            result.expect("failed to parse result");
        assert_eq!(constrains_single.len(), 1);
        let constraint_item = constrains_single.pop().unwrap();
        assert_eq!(
            constraint_item,
            WithdrawalConstraints {
                currency: "twd".into(),
                fee: dec!(0),
                ratio: dec!(0),
                min_amount: dec!(100)
            }
        )
    }
}
