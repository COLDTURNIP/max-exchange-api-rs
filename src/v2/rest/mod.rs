//! Components to interact with the RESTful API: <https://max.maicoin.com/documents/api_list/v2>
//!
//! The parameters of API endpoints are designed to work with `http_types`-compatible HTTP client crates. Each parameter
//! structures provide the following methods:
//!
//! - `to_request(&self)` or `to_request(&self, &crate::Credentials)`: create `http_types::Request` and ready to be sent.
//! - `read_response(http_types::Response)`: parse the response.
//!
//! ```ignore
//! let client = surf::Client::new();
//! let params = GetSomeInformation {...(API parameters)...};
//! let resp = client
//!     .send(params.to_request())
//!     .await
//!     .expect("Error while sending request");
//! let result = GetSomeInformation::read_response(resp.into()).await;
//! let content: ResultContent = result.expect("failed to parse result");
//! // continue work with content
//! ```

pub(crate) mod internal;

mod private;
mod public;

pub use private::*;
pub use public::*;

// =========
// Utilities
// =========

pub(crate) mod api_impl {
    macro_rules! api_url {
        () => {
            "https://max-api.maicoin.com"
        };
        ($endpoint:literal) => {
            concat!(api_url!(), $endpoint)
        };
        (dynamic $fmt:literal, $($args:tt)*) => {
            format!(concat!(api_url!(), $fmt), $($args)*)
        };
    }
    pub(crate) use api_url;

    macro_rules! endpoint_binding {
        (dynamic $sel:ident $gen_endpoint:block) => {
            fn get_url(&self) -> http_types::Url {
                http_types::Url::parse((|$sel: &Self| $gen_endpoint)(self).as_str())
                    .expect("unexpected invalid API URL")
            }
        };
        (fixed $endpoint:literal) => {
            fn get_url(&self) -> http_types::Url {
                http_types::Url::parse(api_url!($endpoint)).expect("unexpected invalid API URL")
            }
        };
    }
    pub(crate) use endpoint_binding;

    macro_rules! convert_to_request {
        (GET) => {
            pub fn to_request(&self) -> http_types::Request {
                crate::v2::rest::internal::make_unauth_get(self)
            }
        };
        (auth GET) => {
            pub fn to_request(&self, credentials: &crate::Credentials) -> http_types::Request {
                crate::v2::rest::internal::make_auth_get(self, credentials)
            }
        };
        (auth POST) => {
            pub fn to_request(&self, credentials: &crate::Credentials) -> http_types::Request {
                crate::v2::rest::internal::make_auth_post(self, credentials)
            }
        };
    }
    pub(crate) use convert_to_request;

    macro_rules! convert_from_response {
        ($resp:ty) => {
            pub async fn read_response(resp: http_types::Response) -> crate::error::Result<$resp> {
                <Self as crate::v2::rest::internal::RestApiBase>::read_response(resp).await
            }
        };
    }
    pub(crate) use convert_from_response;

    macro_rules! impl_api {
        ($api:ty => $resp:ty : GET, $endpoint:literal) => {
            impl $api {
                convert_to_request!(GET);
                convert_from_response!($resp);
            }
            impl crate::v2::rest::internal::RestApiBase for $api {
                endpoint_binding!(fixed $endpoint);
                type Response = $resp;
            }
        };
        ($api:ty => $resp:ty : GET, dynamic $sel:ident $gen_endpoint:block) => {
            impl $api {
                convert_to_request!(GET);
                convert_from_response!($resp);
            }
            #[allow(clippy::redundant_closure_call)]
            impl crate::v2::rest::internal::RestApiBase for $api {
                endpoint_binding!(dynamic $sel $gen_endpoint);
                type Response = $resp;
            }
        };
        ($api:ty => $resp:ty : auth GET, $endpoint:literal) => {
            impl $api {
                convert_to_request!(auth GET);
                convert_from_response!($resp);
            }
            impl crate::v2::rest::internal::RestApiBase for $api {
                endpoint_binding!(fixed $endpoint);
                type Response = $resp;
            }
        };
        ($api:ty => $resp:ty : auth GET, dynamic $sel:ident $gen_endpoint:block) => {
            impl $api {
                convert_to_request!(auth GET);
                convert_from_response!($resp);
            }
            #[allow(clippy::redundant_closure_call)]
            impl crate::v2::rest::internal::RestApiBase for $api {
                endpoint_binding!(dynamic $sel $gen_endpoint);
                type Response = $resp;
            }
        };
        ($api:ty => $resp:ty : auth POST, $endpoint:literal) => {
            impl $api {
                convert_to_request!(auth POST);
                convert_from_response!($resp);
            }
            impl crate::v2::rest::internal::RestApiBase for $api {
                endpoint_binding!(fixed $endpoint);
                type Response = $resp;
            }
        };
        ($api:ty => $resp:ty : auth POST, dynamic $sel:ident $gen_endpoint:block) => {
            impl $api {
                convert_to_request!(auth POST);
                convert_from_response!($resp);
            }
            #[allow(clippy::redundant_closure_call)]
            impl crate::v2::rest::internal::RestApiBase for $api {
                endpoint_binding!(dynamic $sel $gen_endpoint);
                type Response = $resp;
            }
        };
    }
    pub(crate) use impl_api;
}

// ================
// Public constants
// ================

/// The RESTful API base URL.
pub const BASE_URL: &str = api_impl::api_url!();
