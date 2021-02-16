use chrono::serde as chrono_serde;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::common::*;
use crate::v2::rest::api_impl::*;

// ========
// Requests
// ========

/// GET /api/v2/deposits
///
/// Get your deposits history
#[derive(Serialize, Debug)]
pub struct GetDeposits {
    /// Unique currency id, check /api/v2/currencies for available currencies
    pub currency: String,
    /// Target period start (Epoch time in seconds)
    #[serde(
        rename = "from",
        skip_serializing_if = "Option::is_none",
        with = "chrono_serde::ts_seconds_option"
    )]
    pub from_timestamp: Option<DateTime>,
    /// Target period end (Epoch time in seconds)
    #[serde(
        rename = "to",
        skip_serializing_if = "Option::is_none",
        with = "chrono_serde::ts_seconds_option"
    )]
    pub to_timestamp: Option<DateTime>,
    /// Filter deposit state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<DepositState>,
    /// Do pagination & return metadata in header (default true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<bool>,
    /// Paging parameters.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub page_params: Option<PageParams>,
    /// Records to skip, not applied for pagination (default 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
}
impl_api!(GetDeposits => Vec<RespDepositRecord> : auth GET, "/api/v2/deposits");

/// GET /api/v2/deposit
///
/// Get details of a specific deposit
#[derive(Serialize, Debug)]
pub struct GetDepositDetail {
    /// Unique transaction id
    pub txid: String,
}
impl_api!(GetDepositDetail => RespDepositRecord : auth GET, "/api/v2/deposit");

/// GET /api/v2/deposit_addresses
///
/// Get deposit addresses of given currency.
/// Note: The addresses could be empty before generated, please call CreateDepositAddress in that case
#[derive(Serialize, Debug)]
pub struct GetDepositAddresses {
    /// Unique currency id, check /api/v2/currencies for available currencies
    pub currency: String,
    /// Do pagination & return metadata in header (default false)
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub pagination: Option<bool>,
    /// pagination parameters.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub page_params: Option<PageParams>,
    /// Records to skip, not applied for pagination (default 0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
}
impl_api!(GetDepositAddresses => Vec<DepositAddress> : auth GET, "/api/v2/deposit_addresses");

/// POST /api/v2/deposit_addresses
///
/// Greate deposit address of given currency.
/// Note: Address creation is asynchronous, please call GetDepositAddresses later to get generated addresses
#[derive(Serialize, Eq, PartialEq, Debug)]
pub struct CreateDepositAddress {
    /// Unique currency id, check /api/v2/currencies for available currencies
    pub currency: String,
}
impl_api!(CreateDepositAddress => Vec<DepositAddress> : auth POST, "/api/v2/deposit_addresses");

// =========
// Responses
// =========

/// Deposit detail
#[derive(Deserialize, Default, Eq, PartialEq, Debug)]
#[serde(default)]
pub struct RespDepositRecord {
    /// uuid (string, optional): unique deposit id
    pub uuid: String,
    /// currency (string, optional): currency id
    pub currency: String,
    /// currency_version (string, optional): currency version id
    pub currency_version: String,
    /// amount (string, optional): deposit amount
    pub amount: Decimal,
    /// fee (string, optional): deposit fee
    pub fee: Decimal,
    /// txid (string, optional): unique transaction id
    pub txid: String,
    /// created_at (integer, optional): received timestamp (second)
    #[serde(with = "chrono_serde::ts_seconds_option")]
    pub created_at: Option<DateTime>,
    /// confirmations (string, optional): confirmations for crypto currency
    pub confirmations: u64,
    /// updated_at (integer, optional): lastest updated timestamp (second)
    #[serde(with = "chrono_serde::ts_seconds_option")]
    pub updated_at: Option<DateTime>,
    /// state (string, optional): current state
    pub state: DepositState,
}

// ============================
// Inner structures and options
// ============================

/// Possible deposit state
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DepositState {
    Submitting,
    Cancelled,
    Submitted,
    Suspended,
    Rejected,
    Accepted,
    Checking,
    Refunded,
    Suspect,
    RefundCanceled,
    Unknown,
}

impl DepositState {
    pub fn is_unknown(&self) -> bool {
        self == &Self::Unknown
    }
}

impl Default for DepositState {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Deposit address.The addresses could be empty before generated, please call POST /deposit_addresses in that case
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
#[serde(default)]
pub struct DepositAddress {
    /// sn (integer, optional): unique address id
    pub sn: String,
    /// composite_currency (string, optional): currency id
    pub composite_currency: String,
    /// version (string, optional): currency transfer standard, nil if only 1 version supported
    pub version: Option<String>,
    /// currency (string, optional): internal code for the currency
    pub currency: String,
    /// address (string, optional): deposit address, nil when generating or deposit suspended
    pub address: String,
    /// label (string, optional): label of deposit address
    pub label: Option<String>,
    /// type (string, optional): wallet type
    #[serde(rename = "type")]
    pub wallet_type: String,
    /// created_at (integer, optional): created timestamp (second)
    #[serde(with = "chrono_serde::ts_seconds_option")]
    pub created_at: Option<DateTime>,
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
        path_builder.push("deposit");
        path_builder.push(cassette);
        create_test_recording_client(VcrMode::Replay, path_builder.as_path().to_str().unwrap())
            .await
    }

    #[async_std::test]
    async fn get_deposits() {
        let params = GetDeposits {
            currency: "twd".to_string(),
            from_timestamp: None,
            to_timestamp: None,
            state: None,
            pagination: None,
            page_params: None,
            offset: None,
        };
        let resp = create_client("get_deposits.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetDeposits::read_response(resp.into()).await;
        let history: Vec<RespDepositRecord> = result.expect("failed to parse result");
        assert_eq!(history.len(), 27);
    }

    #[async_std::test]
    async fn get_deposit_detail() {
        let params = GetDepositDetail {
            txid: "20201222-2-30388-1024064000298304-1893115".into(),
        };
        let resp = create_client("get_deposit_detail.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetDepositDetail::read_response(resp.into()).await;
        let detail: RespDepositRecord = result.expect("failed to parse result");
        assert_eq!(
            detail,
            RespDepositRecord {
                uuid: "(test erased uuid)".into(),
                currency: "twd".into(),
                currency_version: "twd".into(),
                amount: dec!(50000.0),
                fee: dec!(0),
                txid: "(test erased txid)".into(),
                created_at: Some(Utc.timestamp(1608626791, 0)),
                confirmations: 0,
                updated_at: Some(Utc.timestamp(1608626791, 0)),
                state: DepositState::Accepted,
            }
        );
    }

    #[async_std::test]
    async fn get_deposit_addresses() {
        let params = GetDepositAddresses {
            currency: "btc".into(),
            pagination: None,
            page_params: None,
            offset: None,
        };
        let resp = create_client("get_deposit_addresses.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetDepositAddresses::read_response(resp.into()).await;
        let addr_list: Vec<DepositAddress> = result.expect("failed to parse result");
        assert_eq!(
            addr_list,
            vec![DepositAddress {
                sn: "(test erased sn)".into(),
                composite_currency: "btc".into(),
                version: None,
                currency: "btc".into(),
                address: "(test erased address)".into(),
                label: None,
                wallet_type: "exchange".into(),
                created_at: Some(Utc.timestamp(1599742451, 0)),
            }]
        );
    }

    #[async_std::test]
    async fn create_deposit_addresses() {
        let params = CreateDepositAddress {
            currency: "btc".into(),
        };
        let resp = create_client("create_deposit_addresses.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");

        let result = CreateDepositAddress::read_response(resp.into()).await;
        let addr_list: Vec<DepositAddress> = result.expect("failed to parse result");
        assert_eq!(
            addr_list,
            vec![DepositAddress {
                sn: "(test erased sn)".into(),
                composite_currency: "btc".into(),
                version: None,
                currency: "btc".into(),
                address: "(test erased address)".into(),
                label: None,
                wallet_type: "exchange".into(),
                created_at: Some(Utc.timestamp(1599742451, 0)),
            }]
        );
    }
}
