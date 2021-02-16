use chrono::serde as chrono_serde;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::common::*;
use crate::v2::rest::api_impl::*;

// ========
// Requests
// ========

/// GET /api/v2/withdrawal
///
/// Get details of a specific external withdraw.
#[derive(Serialize, Debug)]
pub struct GetWithdrawal {
    /// Unique withdraw id.
    pub uuid: String,
}
impl_api!(GetWithdrawal => RespWithdrawalDetail : auth GET, "/api/v2/withdrawal");

/// GET /api/v2/withdrawals
///
/// Get your external withdrawals history.
#[derive(Serialize, Debug)]
pub struct GetWithdrawals {
    /// Unique currency id, check /api/v2/currencies for available currencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Target period start (Epoch time in seconds).
    #[serde(rename = "from", skip_serializing_if = "Option::is_none")]
    pub from_timestamp: Option<DateTime>,
    /// Target period end (Epoch time in seconds).
    #[serde(rename = "to", skip_serializing_if = "Option::is_none")]
    pub to_timestamp: Option<DateTime>,
    /// Withdrawal state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<WithdrawalState>,
    /// Do pagination & return metadata in header (default `false`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<bool>,
    /// Pagination parameters, see [`crate::common::PageParams`].
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub page_params: Option<PageParams>,
    /// Records to skip, not applied for pagination (default `0`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
}
impl_api!(GetWithdrawals => Vec<RespWithdrawalDetail> : auth GET, "/api/v2/withdrawals");

/// POST /api/v2/withdrawal
///
/// Submit a withdrawal. IP whitelist for api token is required.
#[derive(Serialize, Debug)]
pub struct CreateWithdrawal {
    /// Unique currency id, check /api/v2/currencies for available currencies.
    pub currency: String,
    /// Unique withdraw address id, check GET /api/v2/withdraw_addresses for available withdraw addresses.
    pub withdraw_address_uuid: String,
    /// Withdraw amount.
    pub amount: Decimal,
}
impl_api!(CreateWithdrawal => RespCreatedWithdraw : auth POST, "/api/v2/withdrawal");

/// GET /api/v2/withdraw_addresses
///
/// Get withdraw addresses by currency.
#[derive(Serialize, Debug)]
pub struct GetWithdrawAddresses {
    /// Unique currency id, check /api/v2/currencies for available currencies.
    pub currency: String,
    /// Do pagination & return metadata in header (default `false`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<bool>,
    /// Pagination parameters, see [`crate::common::PageParams`].
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub page_params: Option<PageParams>,
    /// Records to skip, not applied for pagination (default `0`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
}
impl_api!(GetWithdrawAddresses => Vec<WithdrawAddress> : auth GET, "/api/v2/withdraw_addresses");

// =========
// Responses
// =========

/// Withdrawal detail
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
#[serde(default)]
pub struct RespWithdrawalDetail {
    /// uuid (string, optional): unique withdraw id.
    pub uuid: String,
    /// currency (string, optional): currency id.
    pub currency: String,
    /// currency_version (string, optional): currency version id.
    pub currency_version: String,
    /// amount (string, optional): withdraw amount.
    pub amount: Decimal,
    /// fee (string, optional): withdraw fee.
    pub fee: Decimal,
    /// fee_currency (string, optional): withdraw fee currency.
    pub fee_currency: String,
    /// txid (string, optional): transaction id.
    pub txid: Option<String>,
    /// created_at (integer, optional): created timestamp (second).
    #[serde(with = "chrono_serde::ts_seconds_option")]
    pub created_at: Option<DateTime>,
    /// updated_at (integer, optional): lastest updated timestamp (second).
    #[serde(with = "chrono_serde::ts_seconds_option")]
    pub updated_at: Option<DateTime>,
    /// state (string, optional): current state.
    pub state: WithdrawalState,
}

/// Response of a withdrawal submission
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct RespCreatedWithdraw {
    /// Withdrawal detail.
    #[serde(flatten)]
    pub detail: RespWithdrawalDetail,
    /// type (string, optional): internal/external transfer.
    #[serde(default, rename = "type")]
    pub transaction_direction: TransactionDirection,
    /// transaction_type (string, optional): transaction type.
    pub transaction_type: String,
    /// notes (string, optional): withdraw note.
    pub notes: Option<String>,
    /// sender (object, optional): sender mask email.
    ///
    /// Note: the actual type is string.
    pub sender: Option<String>,
    /// recipient (object, optional): recipient address.
    ///
    /// Note: the actual type is string.
    pub recipient: Option<String>,
}

// ============================
// Inner structures and options
// ============================

/// Possible withdraw states.
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WithdrawalState {
    Submitting,
    Submitted,
    Rejected,
    Accepted,
    Suspect,
    Approved,
    DelistedProcessing,
    Reviewing,
    Processing,
    Retryable,
    Sent,
    Canceled,
    Failed,
    Pending,
    Confirmed,
    Overdue,
    KgiManuallyProcessing,
    KgiManuallyConfirmed,
    KgiPossibleFailed,
    SygnaVerifying,
    Unknown,
}

impl WithdrawalState {
    pub fn is_unknown(&self) -> bool {
        self == &Self::Unknown
    }
}

impl Default for WithdrawalState {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Response of a withdrawal submission.
#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TransactionDirection {
    Internal,
    External,
    Unknown,
}

impl TransactionDirection {
    pub fn is_unknown(&self) -> bool {
        self == &Self::Unknown
    }
}

impl Default for TransactionDirection {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Withdraw address state: unverified/verified/disabled.
#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum WithdrawAddressState {
    Unverified,
    Verified,
    Disabled,
    Unknown,
}

impl WithdrawAddressState {
    pub fn is_unknown(&self) -> bool {
        self == &Self::Unknown
    }
}

impl Default for WithdrawAddressState {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Withdraw address.
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
pub struct WithdrawAddress {
    /// uuid (string, optional): unique withdraw address id.
    pub uuid: String,
    /// currency (string, optional): currency id.
    pub currency: String,
    /// currency_version (string, optional): currency version id.
    pub currency_version: String,
    /// currency_protocol_name (string, optional).
    pub currency_protocol_name: Option<String>,
    /// address (string, optional): address, - for bank account.
    pub address: String,
    /// extra_label (string, optional): descriptive label, null for EOS; bank name for bank account.
    pub extra_label: String,
    /// created_at (integer, optional): created timestamp (second).
    #[serde(with = "chrono_serde::ts_seconds_option")]
    pub created_at: Option<DateTime>,
    /// deleted_at (integer, optional): deleted timestamp (second).
    #[serde(with = "chrono_serde::ts_seconds_option")]
    pub deleted_at: Option<DateTime>,
    /// state (string, optional): bank account state (unverified/verified/disabled), nil for others.
    pub state: Option<WithdrawAddressState>,
    /// sygna_vasp_code (string, optional): sygna vasp code.
    pub sygna_vasp_code: Option<String>,
    /// sygna_user_type (string, optional): sygna user type.
    pub sygna_user_type: Option<String>,
    /// sygna_user_code (string, optional): sygna user code.
    pub sygna_user_code: Option<String>,
    /// is_internal (boolean, optional): internal address or not.
    pub is_internal: Option<bool>,
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
        path_builder.push("withdrawal");
        path_builder.push(cassette);
        create_test_recording_client(VcrMode::Replay, path_builder.as_path().to_str().unwrap())
            .await
    }

    #[async_std::test]
    async fn get_single_withdrawal() {
        let params = GetWithdrawal {
            uuid: "211120074215374658171".into(),
        };
        let resp = create_client("get_single_withdrawal.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result: RespWithdrawalDetail = GetWithdrawal::read_response(resp.into()).await.unwrap();
        assert_eq!(
            result,
            RespWithdrawalDetail {
                uuid: "(test erased uuid)".into(),
                currency: "sol".into(),
                currency_version: "sol".into(),
                amount: dec!(1.0),
                fee: dec!(4.21265078),
                fee_currency: "max".into(),
                txid: Some("(test erased txid)".into()),
                created_at: Some(Utc.timestamp(1637394145, 0)),
                updated_at: Some(Utc.timestamp(1637394215, 0)),
                state: WithdrawalState::Confirmed,
            }
        );
    }

    #[async_std::test]
    async fn get_all_withdrawal() {
        let params = GetWithdrawals {
            currency: Some("sol".into()),
            from_timestamp: None,
            to_timestamp: None,
            state: None,
            pagination: None,
            page_params: None,
            offset: None,
        };
        let resp = create_client("get_all_withdrawal.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result: Vec<RespWithdrawalDetail> =
            GetWithdrawals::read_response(resp.into()).await.unwrap();
        assert_eq!(
            result,
            vec![
                RespWithdrawalDetail {
                    uuid: "(test erased uuid)".into(),
                    currency: "sol".into(),
                    currency_version: "sol".into(),
                    amount: dec!(1.0),
                    fee: dec!(4.21265078),
                    fee_currency: "max".into(),
                    txid: Some("(test erased txid)".into()),
                    created_at: Some(Utc.timestamp(1637394145, 0)),
                    updated_at: Some(Utc.timestamp(1637394215, 0)),
                    state: WithdrawalState::Confirmed,
                },
                RespWithdrawalDetail {
                    uuid: "(test erased uuid)".into(),
                    currency: "sol".into(),
                    currency_version: "sol".into(),
                    amount: dec!(4.32),
                    fee: dec!(4.60232158),
                    fee_currency: "max".into(),
                    txid: Some("(test erased txid)".into()),
                    created_at: Some(Utc.timestamp(1635983513, 0)),
                    updated_at: Some(Utc.timestamp(1635983641, 0)),
                    state: WithdrawalState::Confirmed,
                }
            ]
        );
    }

    #[async_std::test]
    async fn create_withdrawal() {
        let params = CreateWithdrawal {
            currency: "sol".into(),
            withdraw_address_uuid: "f79ad0c7-c321-4234-b0b3-4b3f8445dee9".into(),
            amount: dec!(1),
        };
        let resp = create_client("create_withdrawal.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result: RespCreatedWithdraw =
            CreateWithdrawal::read_response(resp.into()).await.unwrap();
        assert_eq!(
            result,
            RespCreatedWithdraw {
                detail: RespWithdrawalDetail {
                    uuid: "(test erased uuid)".into(),
                    currency: "sol".into(),
                    currency_version: "sol".into(),
                    amount: dec!(1.0),
                    fee: dec!(4.21265078),
                    fee_currency: "max".into(),
                    txid: None,
                    created_at: Some(Utc.timestamp(1637394145, 0)),
                    updated_at: Some(Utc.timestamp(1637394145, 0)),
                    state: WithdrawalState::Submitted,
                },
                transaction_direction: TransactionDirection::External,
                transaction_type: "external_send".into(),
                notes: None,
                sender: Some("(test erased sender)".into()),
                recipient: Some("(test erased recipient)".into()),
            }
        );
    }

    #[async_std::test]
    async fn get_withdraw_addresses() {
        let params = GetWithdrawAddresses {
            currency: "sol".into(),
            pagination: None,
            page_params: None,
            offset: None,
        };
        let resp = create_client("get_withdraw_addresses.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result: Vec<WithdrawAddress> = GetWithdrawAddresses::read_response(resp.into())
            .await
            .unwrap();
        assert_eq!(
            result,
            vec![WithdrawAddress {
                uuid: "(test erased uuid)".into(),
                currency: "sol".into(),
                currency_version: "sol".into(),
                currency_protocol_name: None,
                address: "(test erased address)".to_string(),
                extra_label: "(test erased extra_label)".to_string(),
                created_at: Some(Utc.timestamp(1635983472, 0)),
                deleted_at: None,
                state: None,
                sygna_vasp_code: None,
                sygna_user_type: None,
                sygna_user_code: None,
                is_internal: Some(false),
            }]
        );
    }
}
