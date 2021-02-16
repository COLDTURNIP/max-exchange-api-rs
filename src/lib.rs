//! MaiCoin MAX API client for Rust. This crate is designed to work on any asynchronous runtime.
//!
//! - To RESTful API, the requests and response could be converted to/from `http_types` data types.
//! - To websocket API, the requests and response are designed to be serialize/deserialize by `serde-rs`.
//! - All price and amount representation handled by `rust_decimal`
//! - All timestamp convert to `chrono::DateTime<Utc>`
//!
//! # Examples
//!
//! - Get list of exchange supported currencies via public REST API: `examples/get_currencies.rs`
//! - Private REST API authentication: `examples/rest_auth.rs`
//! - Receiving tickers from public websocket API: `examples/ws_client.rs`
//! - Websocket authentication and channel filtering: `examples/ws_auth.rs`

#![deny(
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    unused_qualifications
)]

use std::env::var as env_var;
use std::ffi::OsStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod error;
pub(crate) mod util;
pub mod v2;

fn clock() -> u64 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    now.as_secs() * 1000 + now.subsec_millis() as u64
}

/// Credentials to access private API. It internally maintains an atomic monotonic clock for payload signing. This
/// implies that the data created from [`Credentials`] must be sent to server as soon as possible.
#[derive(Debug)]
pub struct Credentials {
    pub(crate) access_key: String,
    pub(crate) secret_key: String,
    nonce: AtomicU64,
}

impl Credentials {
    /// Create credential by tokens generated from [API tokens settings](https://max.maicoin.com/api_tokens) .
    pub fn new(access_key: String, secret_key: String) -> Self {
        Self {
            access_key,
            secret_key,
            nonce: AtomicU64::new(clock() - 1),
        }
    }

    /// Given environment variable names, create credentials from their values.
    pub fn from_env(access_var: impl AsRef<OsStr>, secret_var: impl AsRef<OsStr>) -> Self {
        Self {
            access_key: env_var(access_var).unwrap_or_default(),
            secret_key: env_var(secret_var).unwrap_or_default(),
            nonce: AtomicU64::new(clock() - 1),
        }
    }

    pub(crate) fn nonce(&self) -> u64 {
        self.nonce
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |t| {
                Some((t + 1).max(clock()))
            })
            .unwrap()
    }
}

// =====================
// API common components
// =====================

/// Common type definition.
pub mod common {
    use chrono::{DateTime as ChronoDateTime, Utc};
    use serde::{Deserialize, Serialize};

    /// Unique market id, check /api/v2/markets for available markets.
    pub type Symbol = String;

    /// Data type to represent time points. Identical to `chrono::DateTime<Utc>`.
    pub type DateTime = ChronoDateTime<Utc>;

    /// Options for sort list in created time.
    #[derive(Serialize, Copy, Clone, Eq, PartialEq, Debug)]
    #[serde(rename_all = "lowercase")]
    pub enum OrderBy {
        Asc,
        Desc,
    }

    /// Parameters for pagination.
    #[derive(Serialize, Debug)]
    pub struct PageParams {
        /// Page number, applied for pagination (default 1)
        pub page: u64,
        /// Returned limit (1~1000, default 50)
        pub limit: u64,
    }

    impl Default for PageParams {
        fn default() -> Self {
            Self { page: 1, limit: 50 }
        }
    }

    /// Side information used in orders.
    #[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
    #[serde(rename_all = "lowercase")]
    pub enum OrderSide {
        Sell,
        Buy,
        Unknown,
    }

    impl OrderSide {
        pub fn is_unknown(&self) -> bool {
            self == &Self::Unknown
        }
    }

    impl Default for OrderSide {
        fn default() -> Self {
            Self::Unknown
        }
    }

    /// Side information used in trade records.
    #[derive(Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
    #[serde(rename_all = "lowercase")]
    pub enum TradeSide {
        Ask,
        Bid,
        Unknown,
    }

    impl TradeSide {
        pub fn is_unknown(&self) -> bool {
            self == &Self::Unknown
        }
    }

    impl Default for TradeSide {
        fn default() -> Self {
            Self::Unknown
        }
    }
}
