use base64::encode as b64_encode;
use hmac::{Hmac, Mac, NewMac};
use http_types::{
    Body as HTTPBody, Request as HTTPRequest, Response as HTTPResponse, Url as HTTPURL,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::future::Future;
use std::pin::Pin;

use crate::error::*;
use crate::Credentials;

pub(crate) const HEADER_AUTH_ACCESS_KEY: &str = "X-MAX-ACCESSKEY";
pub(crate) const HEADER_AUTH_PAYLOAD: &str = "X-MAX-PAYLOAD";
pub(crate) const HEADER_AUTH_SIGNATURE: &str = "X-MAX-SIGNATURE";

// The out most wrapper of authenticated request parameter body.
//
//   AuthParamsOuterWrapper = AuthParamsInnerWrapper + API path
//
// We generate body signature from this wrapped structure.
#[derive(Serialize)]
pub(super) struct AuthParamsOuterWrapper<'path, 'params, P>
where
    P: Sized + Serialize,
{
    #[serde(flatten)]
    pub inner: AuthParamsInnerWrapper<'params, P>,
    pub path: &'path str,
}

// The wrapper of authenticated request parameter body.
//
//   AuthParamsInnerWrapper = Original API parameter + nonce
//
// This is the final parameters attached in RESTful API call.
#[derive(Serialize)]
pub(super) struct AuthParamsInnerWrapper<'params, P>
where
    P: Sized + Serialize,
{
    #[serde(flatten)]
    pub params: &'params P,
    pub nonce: u64,
}

impl<'path, 'params, P: Serialize> AuthParamsOuterWrapper<'path, 'params, P> {
    pub(super) fn signed_payload(&self, credentials: &Credentials) -> (String, String) {
        let payload = b64_encode(serde_json::to_string(&self).unwrap().as_bytes());
        let mut hmac = Hmac::<Sha256>::new_from_slice(credentials.secret_key.as_bytes()).unwrap();
        hmac.update(payload.as_bytes());
        let signature = format!("{:x}", hmac.finalize().into_bytes());
        (payload, signature)
    }
}

pub(crate) trait RestApiBase: Sized + Serialize {
    fn get_url(&self) -> HTTPURL;

    type Response: DeserializeOwned;
    // async fn fn read_response(mut HTTPResponse) -> Self::Response
    fn read_response(
        mut resp: HTTPResponse,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Response>>>> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum BodyWrapper<Content> {
            Err(ApiErrorWrapper),
            Ok(Content),
        }

        let fut_result = async move {
            resp.body_json::<BodyWrapper<Self::Response>>()
                .await
                .map_err(|parse_err| Error::ReadResponse(Box::new(parse_err.into_inner())))
                .and_then(|parsed| match parsed {
                    BodyWrapper::Ok(result) => Result::Ok(result),
                    BodyWrapper::Err(err_wrapper) => Result::Err(err_wrapper.into()),
                })
        };
        Box::pin(fut_result)
    }
}

pub(crate) fn make_unauth_get(params: &impl RestApiBase) -> HTTPRequest {
    let mut req = HTTPRequest::get(params.get_url());
    req.set_query(params)
        .expect("failed to serialize parameters");
    req
}

pub(crate) fn make_auth_get(params: &impl RestApiBase, credentials: &Credentials) -> HTTPRequest {
    let (url, header_payload, header_signature) = {
        let mut url = params.get_url();
        let params = AuthParamsOuterWrapper {
            path: url.path(),
            inner: AuthParamsInnerWrapper {
                nonce: credentials.nonce(),
                params,
            },
        };
        let qs = serde_qs::to_string(&params.inner).expect("auth parameter serialization failed");
        let (payload, signature) = params.signed_payload(credentials);

        url.set_query(Some(&qs));
        (url, payload, signature)
    };

    let mut req = HTTPRequest::get(url);
    req.insert_header(HEADER_AUTH_ACCESS_KEY, &credentials.access_key);
    req.insert_header(HEADER_AUTH_PAYLOAD, header_payload);
    req.insert_header(HEADER_AUTH_SIGNATURE, header_signature);
    req.insert_header("Content-Type", "application/json");
    req
}

pub(crate) fn make_auth_post(params: &impl RestApiBase, credentials: &Credentials) -> HTTPRequest {
    let url = params.get_url();
    let (body, header_payload, header_signature) = {
        let params = AuthParamsOuterWrapper {
            path: url.path(),
            inner: AuthParamsInnerWrapper {
                nonce: credentials.nonce(),
                params,
            },
        };
        let (payload, signature) = params.signed_payload(credentials);
        let body = HTTPBody::from_json(&params.inner).expect("auth parameter serialization failed");
        (body, payload, signature)
    };

    let mut req = HTTPRequest::post(url);
    req.insert_header(HEADER_AUTH_ACCESS_KEY, &credentials.access_key);
    req.insert_header(HEADER_AUTH_PAYLOAD, header_payload);
    req.insert_header(HEADER_AUTH_SIGNATURE, header_signature);
    req.insert_header("Content-Type", "application/json");
    req.set_body(body);
    req
}
