pub(crate) mod serde {
    use serde::de;
    use serde::{Deserialize, Deserializer};

    pub(crate) fn bool_from_onoff<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.to_lowercase().as_ref() {
            "on" => Ok(true),
            "off" => Ok(false),
            other => Err(de::Error::invalid_value(
                de::Unexpected::Str(other),
                &"ON or OFF",
            )),
        }
    }
}

#[cfg(test)]
pub(crate) mod test_util {
    use std::borrow::Borrow;
    use std::path::PathBuf;

    use lazy_static::lazy_static;
    use serde_json::{json, Value};
    use surf_vcr::{Body as VcrBody, VcrMiddleware, VcrMode};

    use crate::Credentials;

    /// Get path of testing resource
    pub(crate) fn test_resource_path() -> PathBuf {
        [env!("CARGO_MANIFEST_DIR"), "resource", "test"]
            .iter()
            .collect()
    }

    /// Create a Surf HTTP client with the surf-vcr middleware.
    pub(crate) async fn create_test_recording_client(
        mode: VcrMode,
        cassette: &str,
    ) -> surf::Client {
        fn hide_address(obj: &mut serde_json::map::Map<String, Value>) {
            // roughly treat all long strings as wallet address
            for (key, val) in obj.iter_mut() {
                if let Value::String(s) = val {
                    if s.parse::<u64>().is_err() && s.len() > 16 {
                        *s = format!("(test erased {})", key);
                    }
                }
            }
            println!("modified {:?}", obj);
        }

        let vcr = VcrMiddleware::new(mode, cassette)
            .await
            .expect("Failed to create VCR middleware")
            .with_modify_request(|req| {
                req.headers
                    .entry(crate::v2::rest::internal::HEADER_AUTH_ACCESS_KEY.to_lowercase())
                    .and_modify(|val| *val = vec!["(auth key)".into()]);
                req.headers
                    .entry(crate::v2::rest::internal::HEADER_AUTH_PAYLOAD.to_lowercase())
                    .and_modify(|val| *val = vec!["(auth payload)".into()]);
                req.headers
                    .entry(crate::v2::rest::internal::HEADER_AUTH_SIGNATURE.to_lowercase())
                    .and_modify(|val| *val = vec!["(auth signature)".into()]);

                let url_copy = req.url.clone();
                let query: Vec<_> = url_copy
                    .query_pairs()
                    .map(|(key, val)| {
                        let val = if key == "nonce" {
                            std::borrow::Cow::from("(nonce)")
                        } else {
                            val
                        };
                        (key, val)
                    })
                    .collect();
                if !query.is_empty() {
                    req.url.query_pairs_mut().clear();
                    for (k, v) in query {
                        req.url
                            .query_pairs_mut()
                            .append_pair(k.borrow(), v.borrow());
                    }
                }

                match req.body {
                    VcrBody::Str(ref mut body) if !body.is_empty() => {
                        let mut parsed: Value = serde_json::from_str(body).unwrap();
                        if let serde_json::Value::Object(ref mut obj) = parsed {
                            obj.entry("nonce").and_modify(|val| *val = json!(0));
                        }
                        *body = serde_json::to_string(&parsed).unwrap();
                    }
                    _ => {}
                };
            })
            .with_modify_response(|resp| {
                resp.headers
                    .entry("set-cookie".into())
                    .and_modify(|val| *val = vec!["(cookies)".into()]);

                match resp.body {
                    VcrBody::Str(ref mut body) => {
                        println!("raw {:?}", body);
                        let mut parsed: Value = serde_json::from_str(body).unwrap();
                        match parsed {
                            serde_json::Value::Object(ref mut obj) => hide_address(obj),
                            serde_json::Value::Array(ref mut obj_list) => {
                                for item in obj_list.iter_mut() {
                                    if let serde_json::Value::Object(ref mut obj) = item {
                                        hide_address(obj);
                                    }
                                }
                            }
                            _ => {}
                        }
                        *body = serde_json::to_string(&parsed).unwrap();
                    }
                    VcrBody::Bytes(_) => {}
                }
            });
        surf::Client::new().with(vcr)
    }

    lazy_static! {
        pub static ref TEST_CREDENTIALS: Credentials =
            Credentials::from_env("MAX_TEST_ACCESS_KEY", "MAX_TEST_SECRET_KEY");
    }
}
