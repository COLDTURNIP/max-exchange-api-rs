use std::collections::HashMap;

use chrono::serde as chrono_serde;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::common::*;
use crate::v2::rest::api_impl::*;

pub use crate::v2::rest::public::RespVIPLevel;

// ========
// Requests
// ========

/// GET /api/v2/members/profile
///
/// Get personal profile information.
#[derive(Serialize, Debug)]
pub struct GetProfile {}
impl_api!(GetProfile => RespProfile : auth GET, "/api/v2/members/profile");

/// GET /api/v2/members/me
///
/// Get your profile and accounts information.
#[derive(Serialize, Debug)]
pub struct GetProfileAndAccount {}
impl_api!(GetProfileAndAccount => RespProfile : auth GET, "/api/v2/members/me");

/// GET /api/v2/members/vip_level
///
/// Get VIP level info.
#[derive(Serialize, Debug)]
pub struct GetAccountVIPLevel {}
impl_api!(GetAccountVIPLevel => RespAccountVIPInfo : auth GET, "/api/v2/members/vip_level");

/// GET /api/v2/members/accounts/{path_currency}
///
/// Get personal accounts information of a currency.
#[derive(Serialize, Debug)]
pub struct GetAccountOfCurrency {
    /// Get personal accounts information of a currency.
    #[serde(skip)]
    pub path_currency: String,
}
impl_api!(GetAccountOfCurrency => RespAccountCurrencyInfo : auth GET, dynamic params {
    api_url!(dynamic "/api/v2/members/accounts/{}", params.path_currency)
});

/// GET /api/v2/internal_transfers
///
/// Get internal transfers history.
#[derive(Serialize, Debug)]
pub struct GetInternalTransfers {
    /// Unique currency id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Transfer side.
    pub side: InternalTransferSide,
    /// Target period start (Epoch time in seconds).
    #[serde(
        rename = "from",
        with = "chrono_serde::ts_seconds_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub from_timestamp: Option<DateTime>,
    /// Target period end (Epoch time in seconds).
    #[serde(
        rename = "to",
        with = "chrono_serde::ts_seconds_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub to_timestamp: Option<DateTime>,
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
impl_api!(GetInternalTransfers => Vec<RespInternalTransferRecord> : auth GET, "/api/v2/internal_transfers");

/// GET /api/v2/internal_transfer
///
/// Get details of a specific internal transfer.
#[derive(Serialize, Debug)]
pub struct GetInternalTransferByUUID {
    /// Unique internal transfer id.
    pub uuid: String,
}
impl_api!(GetInternalTransferByUUID => RespInternalTransferRecord : auth GET, "/api/v2/internal_transfer");

/// GET /api/v2/rewards
///
/// Get rewards history.
#[derive(Serialize, Debug)]
pub struct GetRewards {
    /// Unique currency id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Target period start (Epoch time in seconds).
    #[serde(
        rename = "from",
        with = "chrono_serde::ts_seconds_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub from_timestamp: Option<DateTime>,
    /// Target period end (Epoch time in seconds).
    #[serde(
        rename = "to",
        with = "chrono_serde::ts_seconds_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub to_timestamp: Option<DateTime>,
    /// Do pagination & return metadata in header (default `true`).
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub pagination: Option<bool>,
    /// Pagination parameters, see [`crate::common::PageParams`].
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub page_params: Option<PageParams>,
    /// Records to skip, not applied for pagination (default `0`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
}
impl_api!(GetRewards => Vec<RewardRecord> : auth GET, "/api/v2/rewards");

/// GET /api/v2/rewards/{path_reward_type}
///
/// Get specific rewards history.
#[derive(Serialize, Debug)]
pub struct GetRewardsOfType {
    /// Reward type.
    #[serde(skip)]
    pub reward_type: RewardType,
    /// Request details.
    #[serde(flatten)]
    pub detail: GetRewards,
}
impl_api!(GetRewardsOfType => Vec<RewardRecord> : auth GET, dynamic params {
    let mut reward_str = String::with_capacity(18);
    for (i, ch) in format!("{:?}", params.reward_type).char_indices() {
        if i > 0 && ch.is_uppercase() {
            reward_str.push('_');
        }
        reward_str.push(ch.to_ascii_lowercase());
    }
    api_url!(dynamic "/api/v2/rewards/{}", reward_str)
});

/// GET /api/v2/yields
///
/// Get specific savings interest history
#[derive(Serialize, Debug)]
pub struct GetSavingInterestHistory {
    /// Unique currency id.
    pub currency: String,
    /// Target period start (Epoch time in seconds).
    #[serde(
        rename = "from",
        with = "chrono_serde::ts_seconds_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub from_timestamp: Option<DateTime>,
    /// Target period end (Epoch time in seconds).
    #[serde(
        rename = "to",
        with = "chrono_serde::ts_seconds_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub to_timestamp: Option<DateTime>,
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
impl_api!(GetSavingInterestHistory => Vec<RewardRecord> : auth GET, "/api/v2/yields");

/// GET /api/v2/max_rewards/yesterday
///
/// Get max rewards yesterday.
#[derive(Serialize, Debug)]
pub struct GetMaxRewardsYesterday {}
impl_api!(GetMaxRewardsYesterday => RespMAXReward : auth GET, "/api/v2/max_rewards/yesterday");

// =========
// Responses
// =========

/// Personal profile information.
///
/// (Represents both `External_V2_Entities_Member` and `External_V2_Entities_MemberAttributes_Profile` in official API document)
#[derive(Deserialize, Eq, PartialEq, Debug, Default)]
#[serde(default)]
pub struct RespProfile {
    /// sn (string, optional): unique serial number.
    pub sn: String,
    /// name (string, optional): user name.
    pub name: String,
    /// email (string, optional): user email.
    pub email: String,
    /// language (string, optional): user language.
    pub language: String,
    /// country_code (string, optional): phone country code.
    pub country_code: String,
    /// phone_set (boolean, optional): valid phone set.
    pub phone_set: Option<bool>,
    /// phone_number (string, optional): user mobile phone number.
    pub phone_number: String,
    /// phone_contact_approved (boolean, optional): is phone_contact approved.
    pub phone_contact_approved: Option<bool>,
    /// status (string, optional): inactivated, activated, or frozen.
    pub status: AccountStatus,
    /// profile_verified (boolean, optional): is user profile verified.
    pub profile_verified: Option<bool>,
    /// kyc_approved (boolean, optional): is kyc approved.
    pub kyc_approved: Option<bool>,
    /// kyc_state (string, optional): member kyc state: unverified, verifying, profile_verifying, verified, rejected.
    pub kyc_state: String,
    /// any_kyc_rejected (boolean, optional): if any of kyc assets or requirements been rejected.
    pub any_kyc_rejected: Option<bool>,
    /// agreement_checked (boolean, optional): if user agree with the latest user agreement.
    pub agreement_checked: Option<bool>,
    /// level (integer, optional): member level.
    pub level: Option<u8>,
    /// vip_level (integer, optional): member VIP level.
    pub vip_level: Option<u8>,
    /// member_type (string, optional): type_guest, type_coin, type_twd.
    pub member_type: MemberType,
    /// bank (`External_V2_Entities_Bank`/`External_V2_Entities_Mcoin_BankAccount`, optional)
    pub bank: Option<BankInfo>,
    /// referral_code (string, optional): referral code.
    pub referral_code: String,
    /// birthday (string, optional): birthday.
    pub birthday: Option<String>,
    /// gender (string, optional): M/F/C (Male/Female/Corporation).
    pub gender: Gender,
    /// nationality (string, optional): nationality.
    pub nationality: Option<String>,
    /// identity_type (string, optional): identity type.
    pub identity_type: Option<String>,
    /// identity_number (string, optional): taiwanese identity number.
    pub identity_number: Option<String>,
    /// individual_verified (boolean, optional): is corporate individuals verified.
    pub individual_verified: Option<bool>,
    /// invoice_carrier_id (string, optional): invoice carrier id.
    pub invoice_carrier_id: Option<String>,
    /// invoice_carrier_type (string, optional): invoice carrier type.
    pub invoice_carrier_type: Option<String>,
    /// is_deleted (boolean, optional): is deleted.
    pub is_deleted: Option<bool>,
    /// is_frozen (boolean, optional): is frozen.
    pub is_frozen: Option<bool>,
    /// is_activated (boolean, optional): is activated.
    pub is_activated: Option<bool>,
    /// is_corporate (boolean, optional): is a corporate account.
    pub is_corporate: Option<bool>,
    // two_factor (object, optional): two factor authentications status.
    // TODO: the exact data type is different from API document
    // pub two_factor: Option<String>,
    /// current_two_factor_type (string, optional): app/sms/nil.
    pub current_two_factor_type: Option<String>,
    /// locked_status_of_2fa (object, optional): time that 2fa lock ends.
    pub locked_status_of_2fa: Option<String>,
    /// documents (External_V2_Entities_MemberDocs, optional).
    pub documents: Option<HashMap<String, String>>,
    /// supplemental_document_type (string, optional): supplemental document type.
    pub supplemental_document_type: Option<String>,
    /// user_agreement_checked (boolean, optional): is user aggreement checked.
    pub user_agreement_checked: Option<bool>,
    /// user_agreement_version (string, optional): which tou version user agree with.
    pub user_agreement_version: Option<String>,
    /// withdrawable (boolean, optional): can user make a withdrawal?
    pub withdrawable: Option<bool>,
    /// accounts (`Array[External_V2_Entities_Account]`, optional).
    pub accounts: Option<Vec<RespAccountCurrencyInfo>>,
}

/// VIP level info.
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
pub struct RespAccountVIPInfo {
    /// current_vip_level (`External_V2_Entities_VipLevel`, optional): current vip level.
    #[serde(rename = "current_vip_level")]
    current: RespVIPLevel,
    /// next_vip_level (`External_V2_Entities_VipLevel`, optional): next vip level.
    #[serde(rename = "next_vip_level")]
    next: RespVIPLevel,
}

/// Personal accounts information of a currency.
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
pub struct RespAccountCurrencyInfo {
    /// currency (string, optional): currency id, e.g. twd, btc, ...
    pub currency: String,
    /// balance (string, optional): available balance
    pub balance: Decimal,
    /// locked (string, optional): locked funds
    pub locked: Decimal,
    /// type (string, optional): wallet type
    #[serde(rename = "type")]
    pub wallet_type: String,
    /// fiat_currency (string, optional): fiat currency id, e.g. twd, usd, ...
    pub fiat_currency: Option<String>,
    /// fiat_balance (string, optional): available balance in fiat currency
    pub fiat_balance: Option<Decimal>,
}

/// Internal transfer.
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
pub struct RespInternalTransferRecord {
    /// uuid (string, optional): unique internal transfer id
    pub uuid: String,
    /// currency (string, optional): currency id
    pub currency: String,
    /// amount (string, optional): transfer amount
    pub amount: Decimal,
    /// created_at (integer, optional): created timestamp (second)
    #[serde(with = "chrono_serde::ts_seconds_option")]
    pub created_at: Option<DateTime>,
    /// state (string, optional): current state
    pub state: String,
    /// from_member (string, optional): from member in email
    pub from_member: String,
    /// to_member (string, optional): to member in email
    pub to_member: String,
}

/// Recent MAX reward.
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
pub struct RespMAXReward {
    /// trading_reward (string, optional): trading reward amount
    pub trading_reward: Decimal,
    /// holding_reward (string, optional): holding reward amount
    pub holding_reward: Decimal,
}

// ============================
// Inner structures and options
// ============================

/// Types of reward.
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RewardType {
    MiningReward,
    HoldingReward,
    TradingReward,
    Commission,
    AirdropReward,
    RedemptionReward,
    VipRebate,
    SavingsInterest,
    Unknown,
}

impl RewardType {
    pub fn is_unknown(&self) -> bool {
        self == &Self::Unknown
    }
}

impl Default for RewardType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Account status.
#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Inactivated,
    Activated,
    Frozen,
    Unknown,
}

impl AccountStatus {
    pub fn is_unknown(&self) -> bool {
        self == &Self::Unknown
    }
}

impl Default for AccountStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Member type.
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub enum MemberType {
    #[serde(rename = "type_guest")]
    Guest,
    #[serde(rename = "type_coin")]
    Coin,
    #[serde(rename = "type_twd")]
    TWD,
    Unknown,
}

impl MemberType {
    pub fn is_unknown(&self) -> bool {
        self == &Self::Unknown
    }
}

impl Default for MemberType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Member bank information
///
/// (Represents both `External_V2_Entities_Bank` and `External_V2_Entities_Mcoin_BankAccount` in official API document)
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
#[serde(default)]
pub struct BankInfo {
    /// bank_code (string, optional): bank code
    pub bank_code: String,
    /// bank_name (string, optional): bank name
    pub bank_name: String,
    /// branch (string, optional): bank branch code
    pub branch: String,
    /// bank_branch_name (string, optional): bank branch name
    pub bank_branch_name: String,
    /// name (string, optional): bank account name
    pub name: Option<String>,
    /// account (string, optional): bank account
    pub account: String,
    /// state (string, optional): bank account state
    pub state: String,
    /// reject_reason (string, optional): bank reject_reason
    pub reject_reason: Option<String>,
    /// intra_bank (boolean, optional): intra bank account
    pub intra_bank: Option<bool>,
    /// bank_branch_active (boolean, optional): bank branch closed
    pub bank_branch_active: Option<bool>,
}

/// Member gender.
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub enum Gender {
    #[serde(rename = "M")]
    Male,
    #[serde(rename = "F")]
    Female,
    #[serde(rename = "C")]
    Corporation,
    Unknown,
}

impl Gender {
    pub fn is_unknown(&self) -> bool {
        self == &Self::Unknown
    }
}

impl Default for Gender {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Internal transfer side, in or out.
#[derive(Serialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum InternalTransferSide {
    In,
    Out,
}

/// Reward record
#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
pub struct RewardRecord {
    /// uuid (string, optional): unique reward id
    pub uuid: String,
    /// type (string, optional): reward type
    #[serde(rename = "type")]
    pub reward_type: RewardType,
    /// currency (string, optional): currency id
    pub currency: String,
    /// amount (string, optional): reward amount
    pub amount: Decimal,
    /// created_at (integer, optional): created timestamp (second)
    #[serde(with = "chrono_serde::ts_seconds_option")]
    pub created_at: Option<DateTime>,
    /// state (string, optional): current state
    pub state: String,
    /// note (string, optional): reward description
    pub note: String,
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
        path_builder.push("misc");
        path_builder.push(cassette);
        create_test_recording_client(VcrMode::Replay, path_builder.as_path().to_str().unwrap())
            .await
    }

    #[async_std::test]
    async fn get_profile() {
        let params = GetProfile {};
        let resp = create_client("get_profile.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetProfile::read_response(resp.into()).await;
        let profile: RespProfile = result.expect("failed to parse result");
        assert_eq!(
            profile,
            RespProfile {
                sn: "(test erased sn)".into(),
                name: "John Doe".into(),
                email: "(test erased email)".to_string(),
                language: "en".into(),
                country_code: "886".into(),
                phone_set: None,
                phone_number: "227221314".into(),
                phone_contact_approved: None,
                status: AccountStatus::Activated,
                profile_verified: Some(true),
                kyc_approved: None,
                kyc_state: "verified".into(),
                any_kyc_rejected: Some(false),
                agreement_checked: Some(true),
                level: Some(2),
                vip_level: Some(0),
                member_type: MemberType::TWD,
                bank: Some(BankInfo {
                    bank_code: "808".into(),
                    bank_name: "玉山商業銀行".into(),
                    branch: "8080000".into(),
                    bank_branch_name: "某某分行".into(),
                    name: None,
                    account: "0000000000000".into(),
                    state: "verified".into(),
                    reject_reason: None,
                    intra_bank: None,
                    bank_branch_active: None
                }),
                referral_code: "58b11077".into(),
                birthday: Some("198****-29".into()),
                gender: Gender::Male,
                nationality: Some("TW".into()),
                identity_type: Some("taiwan_id".into()),
                identity_number: Some("A12****789".into()),
                individual_verified: None,
                invoice_carrier_id: Some("/123ABCD".into()),
                invoice_carrier_type: Some("0a0000".into()),
                is_deleted: None,
                is_frozen: None,
                is_activated: None,
                is_corporate: None,
                current_two_factor_type: Some("app".into()),
                locked_status_of_2fa: None,
                documents: Some(HashMap::from([
                    ("photo_id_front_state".into(), "verified".into()),
                    ("photo_id_back_state".into(), "verified".into()),
                    ("cellphone_bill_state".into(), "verified".into()),
                    ("selfie_with_id_state".into(), "verified".into()),
                ])),
                supplemental_document_type: Some("health_id_card".into()),
                user_agreement_checked: None,
                user_agreement_version: None,
                withdrawable: None,
                accounts: None,
            }
        );
    }

    #[async_std::test]
    async fn get_profile_and_account() {
        let params = GetProfileAndAccount {};
        let resp = create_client("get_profile_and_account.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetProfileAndAccount::read_response(resp.into()).await;
        let profile: RespProfile = result.expect("failed to parse result");
        assert_eq!(
            profile,
            RespProfile {
                sn: "(test erased sn)".into(),
                name: "John Doe".into(),
                email: "(test erased email)".to_string(),
                language: "en".into(),
                country_code: "886".into(),
                phone_set: Some(true),
                phone_number: "227221314".into(),
                phone_contact_approved: None,
                status: AccountStatus::Unknown,
                profile_verified: Some(true),
                kyc_approved: Some(true),
                kyc_state: "verified".into(),
                any_kyc_rejected: Some(false),
                agreement_checked: None,
                level: Some(2),
                vip_level: Some(0),
                member_type: MemberType::TWD,
                bank: Some(BankInfo {
                    bank_code: "808".into(),
                    bank_name: "玉山商業銀行".into(),
                    branch: "8080000".into(),
                    bank_branch_name: "某某分行".into(),
                    name: None,
                    account: "0000000000000".into(),
                    state: "verified".into(),
                    reject_reason: None,
                    intra_bank: None,
                    bank_branch_active: None
                }),
                referral_code: "58b11077".into(),
                birthday: Some("1985-02-29".into()),
                gender: Gender::Male,
                nationality: Some("TW".into()),
                identity_type: Some("taiwan_id".into()),
                identity_number: Some("A123456789".into()),
                individual_verified: None,
                invoice_carrier_id: Some("/123ABCD".into()),
                invoice_carrier_type: Some("0a0000".into()),
                is_deleted: Some(false),
                is_frozen: Some(false),
                is_activated: Some(true),
                is_corporate: Some(false),
                current_two_factor_type: None,
                locked_status_of_2fa: None,
                documents: Some(HashMap::from([
                    ("photo_id_front_state".into(), "verified".into()),
                    ("photo_id_back_state".into(), "verified".into()),
                    ("cellphone_bill_state".into(), "verified".into()),
                    ("selfie_with_id_state".into(), "verified".into()),
                ])),
                supplemental_document_type: Some("health_id_card".into()),
                user_agreement_checked: Some(true),
                user_agreement_version: Some("5.1".into()),
                withdrawable: Some(true),
                accounts: Some(Vec::new()),
            }
        );
    }

    #[async_std::test]
    async fn get_vip_level() {
        let params = GetAccountVIPLevel {};
        let resp = create_client("get_vip_level.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetAccountVIPLevel::read_response(resp.into()).await;
        let level_info: RespAccountVIPInfo = result.expect("failed to parse result");
        assert_eq!(
            level_info,
            RespAccountVIPInfo {
                current: RespVIPLevel {
                    level: 0,
                    minimum_trading_volume: dec!(0),
                    minimum_staking_volume: dec!(0),
                    maker_fee: dec!(0.00045),
                    taker_fee: dec!(0.0015),
                },
                next: RespVIPLevel {
                    level: 1,
                    minimum_trading_volume: dec!(3000000),
                    minimum_staking_volume: dec!(500),
                    maker_fee: dec!(0.00035999999999999997),
                    taker_fee: dec!(0.00135),
                },
            }
        );
    }

    #[async_std::test]
    async fn get_account_of_currency() {
        let params = GetAccountOfCurrency {
            path_currency: "doge".into(),
        };
        let resp = create_client("get_account_of_currency.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetAccountOfCurrency::read_response(resp.into()).await;
        let account_currency: RespAccountCurrencyInfo = result.expect("failed to parse result");
        assert_eq!(
            account_currency,
            RespAccountCurrencyInfo {
                currency: "doge".into(),
                balance: dec!(10000.25),
                locked: dec!(0.0),
                wallet_type: "exchange".into(),
                fiat_currency: None,
                fiat_balance: None,
            }
        );
    }

    #[async_std::test]
    async fn get_internal_transfers() {
        let params = GetInternalTransfers {
            currency: Some("max".into()),
            side: InternalTransferSide::In,
            from_timestamp: None,
            to_timestamp: None,
            pagination: None,
            page_params: None,
            offset: None,
        };
        let resp = create_client("get_internal_transfers.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetInternalTransfers::read_response(resp.into()).await;
        let transfer_history: Vec<RespInternalTransferRecord> =
            result.expect("failed to parse result");
        assert_eq!(
            transfer_history,
            vec![RespInternalTransferRecord {
                uuid: "(test erased uuid)".into(),
                currency: "max".into(),
                amount: dec!(1.0),
                created_at: Some(Utc.timestamp(1605265665, 0)),
                state: "done".into(),
                from_member: "(test erased from_member)".into(),
                to_member: "(test erased to_member)".into()
            }]
        );
    }

    #[async_std::test]
    async fn get_transfers_by_uuid() {
        let params = GetInternalTransferByUUID {
            uuid: "2011131107100357467635".into(),
        };
        let resp = create_client("get_transfers_by_uuid.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetInternalTransferByUUID::read_response(resp.into()).await;
        let record: RespInternalTransferRecord = result.expect("failed to parse result");
        assert_eq!(
            record,
            RespInternalTransferRecord {
                uuid: "(test erased uuid)".into(),
                currency: "max".into(),
                amount: dec!(1.0),
                created_at: Some(Utc.timestamp(1605265665, 0)),
                state: "done".into(),
                from_member: "(test erased from_member)".into(),
                to_member: "(test erased to_member)".into()
            }
        );
    }

    #[async_std::test]
    async fn get_rewards() {
        let params = GetRewards {
            currency: Some("max".into()),
            from_timestamp: Some(Utc.timestamp(1637316000, 0)),
            to_timestamp: None,
            pagination: None,
            page_params: None,
            offset: None,
        };
        let resp = create_client("get_rewards.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetRewards::read_response(resp.into()).await;
        let reward_history: Vec<RewardRecord> = result.expect("failed to parse result");
        assert_eq!(
            reward_history,
            vec![RewardRecord {
                uuid: "(test erased uuid)".into(),
                reward_type: RewardType::HoldingReward,
                currency: "max".into(),
                amount: dec!(6.21724144),
                created_at: Some(Utc.timestamp(1637346829, 0)),
                state: "done".into(),
                note: "(test erased note)".into()
            }]
        );
    }

    #[async_std::test]
    async fn get_rewards_of_type() {
        let params = GetRewardsOfType {
            reward_type: RewardType::HoldingReward,
            detail: GetRewards {
                currency: Some("max".into()),
                from_timestamp: Some(Utc.timestamp(1637316000, 0)),
                to_timestamp: None,
                pagination: None,
                page_params: None,
                offset: None,
            },
        };
        let resp = create_client("get_rewards_of_type.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetRewardsOfType::read_response(resp.into()).await;
        let reward_history: Vec<RewardRecord> = result.expect("failed to parse result");
        assert_eq!(
            reward_history,
            vec![RewardRecord {
                uuid: "(test erased uuid)".into(),
                reward_type: RewardType::HoldingReward,
                currency: "max".into(),
                amount: dec!(6.21724144),
                created_at: Some(Utc.timestamp(1637346829, 0)),
                state: "done".into(),
                note: "(test erased note)".into()
            }]
        );
    }

    #[async_std::test]
    async fn get_saving_interest_history() {
        let params = GetSavingInterestHistory {
            currency: "usdt".to_string(),
            from_timestamp: Some(Utc.timestamp(1634724000, 0)),
            to_timestamp: None,
            pagination: None,
            page_params: None,
            offset: None,
        };
        let resp = create_client("get_saving_interest_history.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetSavingInterestHistory::read_response(resp.into()).await;
        let interest_history: Vec<RewardRecord> = result.expect("failed to parse result");
        assert_eq!(
            interest_history,
            vec![
                RewardRecord {
                    uuid: "(test erased uuid)".to_string(),
                    reward_type: RewardType::SavingsInterest,
                    currency: "usdt".to_string(),
                    amount: dec!(0.00005154),
                    created_at: Some(Utc.timestamp(1635711201, 0)),
                    state: "done".to_string(),
                    note: "(test erased note)".to_string()
                },
                RewardRecord {
                    uuid: "(test erased uuid)".to_string(),
                    reward_type: RewardType::SavingsInterest,
                    currency: "usdt".to_string(),
                    amount: dec!(0.03194253),
                    created_at: Some(Utc.timestamp(1634760738, 0)),
                    state: "done".to_string(),
                    note: "(test erased note)".to_string()
                }
            ]
        );
    }

    #[async_std::test]
    async fn get_max_rewards_yesterday() {
        let params = GetMaxRewardsYesterday {};
        let resp = create_client("get_max_rewards_yesterday.yaml")
            .await
            .send(params.to_request(&TEST_CREDENTIALS))
            .await
            .expect("Error while sending request");
        let result = GetMaxRewardsYesterday::read_response(resp.into()).await;
        let reward_info: RespMAXReward = result.expect("failed to parse result");
        assert_eq!(
            reward_info,
            RespMAXReward {
                trading_reward: dec!(0.0),
                holding_reward: dec!(6.21724144),
            }
        );
    }
}
